use std::sync::{Arc, Mutex};

use anyhow::Result;
use gpui::{div, rgb, ParentElement, Render, Styled, View, ViewContext, VisualContext};
use hyprland::{
    data::{Monitors, Workspace as HWorkspace, Workspaces as HWorkspaces},
    event_listener::EventListener,
    shared::{HyprData, HyprDataActive, HyprDataVec, WorkspaceType},
};
use tokio::{
    sync::broadcast::{channel, Sender},
    task::spawn_blocking,
};
use tracing::{debug, error, info};

use ui::h_flex;

#[derive(Debug, Clone)]
struct Workspace {
    id: i32,
    name: String,
    monitor_id: Option<usize>,
    visibility: Visibility,
    windows: u16,
}

/// Indicates workspace visibility.
/// Visible workspaces have a boolean flag to indicate if they are focused.
#[derive(Debug, Clone, Copy)]
enum Visibility {
    Visible { focused: bool },
    Hidden,
}

impl Visibility {
    fn visible() -> Self {
        Self::Visible { focused: false }
    }

    fn focused() -> Self {
        Self::Visible { focused: true }
    }

    fn is_focused(self) -> bool {
        if let Self::Visible { focused } = self {
            focused
        } else {
            false
        }
    }
}

fn get_workspace(name: &str, active: Option<&Workspace>) -> Option<Workspace> {
    HWorkspaces::get()
        .expect("Failed to get workspaces")
        .into_iter()
        .find_map(|w| {
            if w.name == name {
                let vis = Visibility::from((&w, active.map(|w| w.name.as_ref()), &|w| {
                    create_is_visible()(w)
                }));

                Some(Workspace::from((vis, w)))
            } else {
                None
            }
        })
}

fn send_focus_change(
    prev_workspace: &mut Option<Workspace>,
    workspace: Workspace,
    tx: &Sender<WorkspaceMessage>,
) {
    tx.send(WorkspaceMessage::Focus {
        old: prev_workspace.take(),
        new: workspace.clone(),
    })
    .expect("failed to send message");

    tx.send(WorkspaceMessage::Urgent {
        id: workspace.id,
        urgent: false,
    })
    .expect("failed to send message");

    prev_workspace.replace(workspace);
}

fn create_is_visible() -> impl Fn(&HWorkspace) -> bool {
    let monitors = Monitors::get().map_or(Vec::new(), HyprDataVec::to_vec);

    move |w| monitors.iter().any(|m| m.id == w.monitor_id)
}

fn get_active_workspace() -> Result<Workspace> {
    let w = hyprland::data::Workspace::get_active()
        .map(|w| Workspace::from((Visibility::focused(), w)))?;
    Ok(w)
}

fn get_workspace_name(name: WorkspaceType) -> String {
    match name {
        WorkspaceType::Regular(name) => name,
        WorkspaceType::Special(name) => name.unwrap_or_default(),
    }
}

impl From<(Visibility, HWorkspace)> for Workspace {
    fn from((visibility, workspace): (Visibility, HWorkspace)) -> Self {
        Self {
            id: workspace.id,
            name: workspace.name,
            monitor_id: Some(workspace.monitor_id as usize),
            windows: workspace.windows,
            visibility,
        }
    }
}

impl<'a, 'f, F> From<(&'a HWorkspace, Option<&str>, F)> for Visibility
where
    F: FnOnce(&'f HWorkspace) -> bool,
    'a: 'f,
{
    fn from((workspace, active_name, is_visible): (&'a HWorkspace, Option<&str>, F)) -> Self {
        if Some(workspace.name.as_str()) == active_name {
            Self::focused()
        } else if is_visible(workspace) {
            Self::visible()
        } else {
            Self::Hidden
        }
    }
}

#[derive(Debug, Clone)]
enum WorkspaceMessage {
    Init(Vec<Workspace>),
    Add(Workspace),
    Remove(i32),
    Move(Workspace),

    Focus {
        old: Option<Workspace>,
        new: Workspace,
    },
    Rename {
        id: i32,
        name: String,
    },

    Urgent {
        id: i32,
        urgent: bool,
    },
}

pub struct Workspaces {
    // TODO: make this a sorted map
    workspaces: Vec<Workspace>,
    tx: Sender<WorkspaceMessage>,
}

impl Workspaces {
    pub fn new<V: 'static>(cx: &mut ViewContext<V>) -> View<Self> {
        cx.new_view(|cx| {
            let (tx, _) = channel(16);

            let mut instance = Self {
                workspaces: vec![],
                tx,
            };

            instance.init_workspaces();
            instance.listen_events();
            instance.handle_event(cx);

            instance
        })
    }

    fn init_workspaces(&self) {
        let tx = self.tx.clone();

        spawn_blocking(move || {
            let active_id = HWorkspace::get_active().ok().map(|active| active.name);
            let is_visible = create_is_visible();

            let workspaces: Vec<Workspace> = HWorkspaces::get()
                .expect("Failed to get workspaces")
                .into_iter()
                .map(|w| {
                    let vis = Visibility::from((&w, active_id.as_deref(), &is_visible));

                    Workspace::from((vis, w))
                })
                .collect();

            tx.send(WorkspaceMessage::Init(workspaces))
                .expect("failed to send message");
        });
    }

    fn listen_events(&self) {
        info!("Starting Hyprland event listener");

        let tx = self.tx.clone();
        spawn_blocking(move || {
            let mut event_listener = EventListener::new();

            let lock = Arc::new(Mutex::new(()));

            let active = get_active_workspace().expect("Failed to get active workspace");
            let active = Arc::new(Mutex::new(Some(active)));

            {
                let tx = tx.clone();
                let lock = lock.clone();
                let active = active.clone();

                event_listener.add_workspace_added_handler({
                    move |event_data| {
                        let _lock = lock.lock().unwrap();
                        let workspace_type = event_data.name;
                        debug!("Added workspace: {:?}", workspace_type);

                        let workspace_name = get_workspace_name(workspace_type);
                        let prev_workspace = active.lock().unwrap();

                        let workspace = get_workspace(&workspace_name, prev_workspace.as_ref());

                        if let Some(workspace) = workspace {
                            tx.send(WorkspaceMessage::Add(workspace))
                                .expect("failed to send message");
                        }
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();
                let active = active.clone();

                event_listener.add_workspace_changed_handler({
                    move |event_data| {
                        let _lock = lock.lock().unwrap();

                        let mut prev_workspace = active.lock().unwrap();
                        let workspace_type = event_data.name;

                        debug!(
                            "Received workspace change: {:?} -> {workspace_type:?}",
                            prev_workspace.as_ref().map(|w| &w.id)
                        );

                        let workspace_name = get_workspace_name(workspace_type);
                        let workspace = get_workspace(&workspace_name, prev_workspace.as_ref());

                        workspace.map_or_else(
                            || {
                                error!("Unable to locate workspace");
                            },
                            |workspace| {
                                if !workspace.visibility.is_focused() {
                                    send_focus_change(&mut prev_workspace, workspace, &tx);
                                }
                            },
                        );
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();
                let active = active.clone();

                event_listener.add_workspace_moved_handler({
                    move |event_data| {
                        let _lock = lock.lock().unwrap();
                        let workspace_type = event_data.name;
                        debug!("Received workspace move: {:?}", workspace_type);

                        let mut prev_workspace = active.lock().unwrap();

                        let workspace_name = get_workspace_name(workspace_type);
                        let workspace = get_workspace(&workspace_name, prev_workspace.as_ref());

                        if let Some(workspace) = workspace {
                            tx.send(WorkspaceMessage::Move(workspace.clone()))
                                .expect("failed to send message");

                            if !workspace.visibility.is_focused() {
                                send_focus_change(&mut prev_workspace, workspace, &tx);
                            }
                        }
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_renamed_handler(move |event_data| {
                    let _lock = lock.lock().unwrap();
                    debug!("Received workspace rename: {:?}", event_data);

                    tx.send(WorkspaceMessage::Rename {
                        id: event_data.id,
                        name: event_data.name,
                    })
                    .expect("failed to send message");
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_workspace_deleted_handler({
                    move |event_data| {
                        let _lock = lock.lock().unwrap();
                        debug!("Received workspace deleted: {:?}", event_data);

                        tx.send(WorkspaceMessage::Remove(event_data.id))
                            .expect("failed to send message");
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();
                let active = active.clone();

                event_listener.add_active_monitor_changed_handler({
                    move |event_data| {
                        let _lock = lock.lock().unwrap();
                        let workspace_type = event_data.workspace_name.unwrap();
                        let mut prev_workspace = active.lock().unwrap();

                        debug!(
                            "Received active monitor change: {workspace_type:?} -> {:?}",
                            prev_workspace.as_ref().map(|w| &w.name)
                        );

                        let workspace_name = get_workspace_name(workspace_type);
                        let workspace = get_workspace(&workspace_name, prev_workspace.as_ref());

                        if let Some((false, workspace)) =
                            workspace.map(|w| (w.visibility.is_focused(), w))
                        {
                            send_focus_change(&mut prev_workspace, workspace, &tx);
                        } else {
                            error!("Unable to locate workspace: {workspace_name}");
                        }
                    }
                });
            }

            {
                let tx = tx.clone();
                let lock = lock.clone();

                event_listener.add_urgent_state_changed_handler({
                    move |address| {
                        let _lock = lock.lock().unwrap();
                        debug!("Received workspace urgent: {:?}", address);

                        let clients = match hyprland::data::Clients::get() {
                            Ok(clients) => clients,
                            Err(e) => {
                                error!("Failed to get clients: {e}");
                                return;
                            }
                        };
                        clients.iter().find(|c| c.address == address).map_or_else(
                            || {
                                error!("Unable to locate client: {address}");
                            },
                            |c| {
                                tx.send(WorkspaceMessage::Urgent {
                                    id: c.workspace.id,
                                    urgent: true,
                                })
                                .expect("failed to send message");
                            },
                        )
                    }
                });
            }

            event_listener
                .start_listener()
                .expect("failed to start listener");
        });
    }

    fn handle_event(&mut self, cx: &mut ViewContext<Self>) {
        info!("Starting Hyprland event handler");

        let mut rx = self.tx.subscribe();
        cx.spawn(|this, mut cx| async move {
            while let Ok(msg) = rx.recv().await {
                debug!("workspace event: {:?}", msg);
                match msg {
                    WorkspaceMessage::Init(workspaces) => {
                        this.update(&mut cx, |this: &mut Self, cx| {
                            this.workspaces = workspaces;
                            cx.notify();
                        })
                        .ok();
                    }
                    WorkspaceMessage::Add(workspace) => {
                        this.update(&mut cx, |this: &mut Self, cx| {
                            this.workspaces.push(workspace);
                            cx.notify();
                        })
                        .ok();
                    }
                    WorkspaceMessage::Remove(id) => {
                        this.update(&mut cx, |this: &mut Self, cx| {
                            this.workspaces.retain(|w| w.id != id);
                            cx.notify();
                        })
                        .ok();
                    }
                    WorkspaceMessage::Move(_) => (),
                    WorkspaceMessage::Focus { old, new } => {
                        this.update(&mut cx, |this: &mut Self, cx| {
                            this.workspaces.iter_mut().for_each(|w| {
                                if w.id == new.id {
                                    w.visibility = Visibility::focused();
                                } else if let Some(old) = &old {
                                    if w.id == old.id {
                                        w.visibility = Visibility::visible();
                                    }
                                }
                            });
                            cx.notify();
                        })
                        .ok();
                    }
                    WorkspaceMessage::Rename { id, name } => (),
                    WorkspaceMessage::Urgent { id, urgent } => (),
                }
            }
        })
        .detach();
    }
}

impl Render for Workspaces {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
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
                        if w.visibility.is_focused() {
                            rgb(0xd8bfd8)
                        } else {
                            rgb(0xeff1f5)
                        }
                    })
                    .child(w.id.to_string())
            }))
    }
}
