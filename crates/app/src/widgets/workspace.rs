use std::sync::{Arc, Mutex};

use hyprland::{
    event_listener::EventListener,
    shared::{HyprData, HyprDataActive, HyprDataVec},
};
use tokio::{
    sync::broadcast::{channel, Receiver, Sender},
    task::spawn_blocking,
};
use tracing::{debug, info};

use ui::{
    div, h_flex, prelude::Window, rgb, AppContext, Context, Entity, EventEmitter, IntoElement,
    ParentElement, Render, Styled,
};

#[derive(Debug, Clone)]
struct Workspace {
    pub id: i32,
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub monitor_id: Option<usize>,
    pub active: bool,
    #[allow(dead_code)]
    pub windows: u16,
}

fn get_workspaces() -> Vec<Workspace> {
    let active = hyprland::data::Workspace::get_active().ok();
    let monitors = hyprland::data::Monitors::get()
        .map(|m| m.to_vec())
        .unwrap_or_default();
    let mut workspaces = hyprland::data::Workspaces::get()
        .map(|w| w.to_vec())
        .unwrap_or_default();

    workspaces.sort_by_key(|w| w.id);

    workspaces
        .into_iter()
        .flat_map(|w| {
            if w.id < 0 {
                vec![Workspace {
                    id: w.id,
                    name: w
                        .name
                        .split(":")
                        .last()
                        .map_or_else(|| "".to_string(), |s| s.to_owned()),
                    monitor_id: Some(w.monitor_id as usize),
                    active: monitors.iter().any(|m| m.special_workspace.id == w.id),
                    windows: w.windows,
                }]
            } else {
                vec![Workspace {
                    id: w.id,
                    name: w.name.clone(),
                    monitor_id: Some(w.monitor_id as usize),
                    active: Some(w.id) == active.as_ref().map(|w| w.id),
                    windows: w.windows,
                }]
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
enum WorkspaceMessage {
    Changed,
}

pub struct Workspaces {
    workspaces: Vec<Workspace>,
    tx: Sender<WorkspaceMessage>,
    #[allow(dead_code)]
    rx: Receiver<WorkspaceMessage>,
}

impl EventEmitter<WorkspaceMessage> for Workspaces {}

impl Workspaces {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let (tx, rx) = channel(16);
            let mut instance = Self {
                workspaces: get_workspaces(),
                tx,
                rx,
            };

            instance.listen_events();
            instance.handle_event(cx);

            instance
        })
    }

    fn listen_events(&self) {
        info!("Starting Hyprland event listener");

        let tx = self.tx.clone();
        spawn_blocking(move || {
            let mut event_listener = EventListener::new();
            let lock = Arc::new(Mutex::new(()));

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_changed_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("workspace change: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_added_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("workspace add: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_moved_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("workspace delete: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_deleted_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("workspace moved: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_changed_special_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("special workspace changed: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_special_removed_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("special workspace removed: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_active_monitor_changed_handler({
                    move |e| {
                        let _lock = lock.lock().unwrap();
                        debug!("active monitor changed: {:?}", e);

                        tx.send(WorkspaceMessage::Changed)
                            .expect("failed to send message");
                    }
                });
            }

            event_listener
                .start_listener()
                .expect("failed to start listener");
        });
    }

    fn handle_event(&mut self, cx: &mut Context<Self>) {
        let mut rx = self.tx.subscribe();
        cx.spawn(|this, mut cx| async move {
            while let Ok(msg) = rx.recv().await {
                debug!("workspace event: {:?}", msg);
                match msg {
                    WorkspaceMessage::Changed => {
                        this.update(&mut cx, |this: &mut Self, cx| {
                            this.workspaces = get_workspaces();
                            cx.notify();
                        })
                        .ok();
                    }
                }
            }
        })
        .detach();
    }
}

impl Render for Workspaces {
    fn render(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        debug!("render workspaces");

        h_flex()
            .gap_x_3()
            .justify_between()
            .children(self.workspaces.iter().map(|w| {
                div()
                    .size_7()
                    .border_2()
                    .rounded_xl()
                    .px_2()
                    .border_color(rgb(0xffc0cb))
                    .bg({
                        if w.active {
                            rgb(0xd8bfd8)
                        } else {
                            rgb(0xeff1f5)
                        }
                    })
                    .child(w.name.to_string())
            }))
    }
}
