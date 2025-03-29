use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use futures::{SinkExt, StreamExt, channel::mpsc::Sender};
use hyprland::{
    dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial},
    event_listener::{AsyncEventListener, WorkspaceEventData, WorkspaceMovedEventData},
    shared::{HyprData, HyprDataActive},
};
use tracing::{debug, info};

use ui::prelude::*;

#[derive(Debug, Clone)]
struct Workspace {
    pub id: i32,
    pub name: String,
    // pub monitor_id: Option<usize>,
    pub active: bool,
    pub visible: bool,
    // pub windows: u16,
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let w = self.clone();
        Button::new()
            .label(w.name)
            .size(28.)
            .border(2.)
            .rounded(10.)
            .when(w.active, |this| this.border_color(rgb(0xffc0cb)))
            .on_click(move |_, _, _| {
                Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
                    w.id,
                )))
                .ok();
            })
    }
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ChangeWorkspace(i32),
    AddWorkspace(i32),
    MoveWorkspace(i32),
    RemoveWorkspace(i32),
}

pub struct Workspaces {
    workspaces: Vec<Workspace>,
    active: usize,
}

impl Workspaces {
    fn init() -> Self {
        let mut workspaces = vec![Workspace {
            id: 0,
            name: "S".to_string(),
            active: false,
            visible: false,
        }];
        for i in 1..11 {
            workspaces.push(Workspace {
                id: i,
                name: format!("{}", i),
                active: false,
                visible: false,
            });
        }
        let active: usize = hyprland::data::Workspace::get_active().unwrap().id as usize;
        let visibles = hyprland::data::Workspaces::get().unwrap();
        visibles.iter().for_each(|w| {
            let mut id = w.id;
            if id < 0 {
                id = 0;
            }
            workspaces[id as usize].visible = true;
        });
        workspaces[active].active = true;

        Self { workspaces, active }
    }

    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        let (tx, mut rx) = futures::channel::mpsc::channel(100);
        listen_events(tx.clone());

        let res = cx.new(|_| Self::init());

        cx.spawn({
            let cnt = Arc::new(AtomicUsize::new(0));
            let this = res.clone();

            async move |_, cx| {
                while let Some(msg) = rx.next().await {
                    cnt.fetch_add(1, Ordering::SeqCst);
                    debug!(
                        "Received message: {:?}, cnt: {}",
                        msg,
                        cnt.load(Ordering::SeqCst)
                    );
                    match msg {
                        Message::ChangeWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            this.update(cx, |this, cx| {
                                this.workspaces[this.active].active = false;
                                let active = id as usize;
                                this.workspaces[active].active = true;
                                this.workspaces[active].visible = true;
                                this.active = active;
                                cx.notify();
                            })
                            .ok();
                        }
                        Message::AddWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            this.update(cx, |this, _cx| {
                                let active = id as usize;
                                this.workspaces[active].visible = true;
                            })
                            .ok();
                        }
                        Message::MoveWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            this.update(cx, |this, _cx| {
                                this.workspaces[this.active].active = false;
                                let active = id as usize;
                                this.workspaces[active].active = true;
                                this.workspaces[active].visible = true;
                                this.active = active;
                            })
                            .ok();
                        }
                        Message::RemoveWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            this.update(cx, |this, cx| {
                                let active = id as usize;
                                this.workspaces[active].visible = false;
                                cx.notify();
                            })
                            .ok();
                        }
                    }

                    Timer::after(Duration::from_millis(10)).await;
                }
                anyhow::Ok(())
            }
        })
        .detach();

        res
    }
}

fn listen_events(tx: Sender<Message>) {
    info!("Listening for hyprland events...");

    let tx = tx.clone();
    // FIXME: maybe remove tokio, using cx.spawn, that can reduce memory usage,
    // but cx.spawn will cause stuck issue, which i don't know why.
    tokio::spawn(async move {
        let mut event_listener = AsyncEventListener::new();

        {
            event_listener.add_workspace_changed_handler({
                let tx = tx.clone();
                move |e| {
                    debug!("Workspace changed: {}", e.id);
                    let mut tx = tx.clone();
                    Box::pin(async move {
                        tx.send(Message::ChangeWorkspace(e.id)).await.unwrap();
                    })
                }
            });
        }

        {
            event_listener.add_workspace_added_handler({
                let tx = tx.clone();
                move |evt: WorkspaceEventData| {
                    debug!("Workspace added: {}", evt.id);
                    let mut tx = tx.clone();
                    Box::pin(async move {
                        tx.send(Message::AddWorkspace(evt.id)).await.unwrap();
                    })
                }
            });
        }

        {
            event_listener.add_workspace_moved_handler({
                let tx = tx.clone();
                move |evt: WorkspaceMovedEventData| {
                    debug!("Workspace moved: {}", evt.id);
                    let mut tx = tx.clone();
                    Box::pin(async move {
                        tx.send(Message::MoveWorkspace(evt.id)).await.unwrap();
                    })
                }
            });
        }

        {
            event_listener.add_workspace_deleted_handler({
                let tx = tx.clone();
                move |evt: WorkspaceEventData| {
                    debug!("Workspace deleted: {}", evt.id);
                    let mut tx = tx.clone();
                    Box::pin(async move {
                        let _ = tx.send(Message::RemoveWorkspace(evt.id)).await.unwrap();
                    })
                }
            });
        }

        event_listener.start_listener_async().await.unwrap();
    });
}

impl Render for Workspaces {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let workspaces = self.workspaces.clone();
        h_flex().justify_between().children(
            workspaces
                .into_iter()
                .filter(|w| w.visible)
                .map(|w| cx.new(|_| w)),
        )
    }
}
