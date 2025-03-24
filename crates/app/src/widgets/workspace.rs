use hyprland::{
    dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial},
    event_listener::{AsyncEventListener, WorkspaceEventData, WorkspaceMovedEventData},
    shared::{HyprData, HyprDataActive},
};
use tokio::sync::broadcast::{Receiver, Sender, channel};
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
        let (tx, _): (Sender<Message>, Receiver<Message>) = channel(16);
        listen_events(tx.clone());

        cx.new(|cx| {
            let instance = Self::init();

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let entity = this.upgrade().unwrap();
                let mut rx = tx.subscribe();
                while let Ok(msg) = rx.recv().await {
                    debug!("Received message: {:?}", msg);
                    match msg {
                        Message::ChangeWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            entity
                                .update(cx, |this, cx| {
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
                            entity
                                .update(cx, |this, cx| {
                                    let active = id as usize;
                                    this.workspaces[active].visible = true;
                                    cx.notify();
                                })
                                .ok();
                        }
                        Message::MoveWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            entity
                                .update(cx, |this, cx| {
                                    this.workspaces[this.active].active = false;
                                    let active = id as usize;
                                    this.workspaces[active].active = true;
                                    this.workspaces[active].visible = true;
                                    this.active = active;
                                    cx.notify();
                                })
                                .ok();
                        }
                        Message::RemoveWorkspace(mut id) => {
                            if id < 0 {
                                id = 0;
                            }
                            entity
                                .update(cx, |this, cx| {
                                    let active = id as usize;
                                    this.workspaces[active].visible = false;
                                    cx.notify();
                                })
                                .ok();
                        }
                    }
                }
            })
            .detach();

            instance
        })
    }
}

fn listen_events(tx: Sender<Message>) {
    info!("Listening for hyprland events...");

    let tx = tx.clone();
    tokio::spawn(async move {
        let mut event_listener = AsyncEventListener::new();

        {
            event_listener.add_workspace_changed_handler({
                let tx = tx.clone();
                move |e| {
                    debug!("Workspace changed: {}", e.id);
                    let tx = tx.clone();
                    Box::pin(async move {
                        let _ = tx.send(Message::ChangeWorkspace(e.id));
                    })
                }
            });
        }

        {
            event_listener.add_workspace_added_handler({
                let tx = tx.clone();
                move |evt: WorkspaceEventData| {
                    debug!("Workspace added: {}", evt.id);
                    let tx = tx.clone();
                    Box::pin(async move {
                        let _ = tx.send(Message::AddWorkspace(evt.id));
                    })
                }
            });
        }

        {
            event_listener.add_workspace_moved_handler({
                let tx = tx.clone();
                move |evt: WorkspaceMovedEventData| {
                    debug!("Workspace moved: {}", evt.id);
                    let tx = tx.clone();
                    Box::pin(async move {
                        let _ = tx.send(Message::MoveWorkspace(evt.id));
                    })
                }
            });
        }

        {
            event_listener.add_workspace_deleted_handler({
                let tx = tx.clone();
                move |evt: WorkspaceEventData| {
                    debug!("Workspace deleted: {}", evt.id);
                    let tx = tx.clone();
                    Box::pin(async move {
                        let _ = tx.send(Message::RemoveWorkspace(evt.id));
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
