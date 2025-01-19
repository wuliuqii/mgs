use std::time::Duration;

use gpui::{div, rgb, AppContext, Context, Model, ParentElement, Render, Styled};
use hyprland::{
    event_listener::AsyncEventListener,
    shared::{HyprData, HyprDataActive, HyprDataVec},
};
use tokio::sync::mpsc;
use tracing::info;

use crate::{status_bar::StatusItemView, ui::stack::h_flex};

const UPDATE_DEBOUNCE: Duration = Duration::from_millis(100);

struct Workspace {
    pub id: i32,
    pub name: String,
    pub monitor_id: Option<usize>,
    pub active: bool,
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

    let mut current: usize = 1;

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
                let missing: usize = w.id as usize - current;
                let mut res = Vec::with_capacity(missing + 1);
                for i in 0..missing {
                    res.push(Workspace {
                        id: (current + i) as i32,
                        name: (current + i).to_string(),
                        monitor_id: None,
                        active: false,
                        windows: 0,
                    });
                }
                current += missing + 1;
                res.push(Workspace {
                    id: w.id,
                    name: w.name.clone(),
                    monitor_id: Some(w.monitor_id as usize),
                    active: Some(w.id) == active.as_ref().map(|w| w.id),
                    windows: w.windows,
                });

                res
            }
        })
        .collect()
}

#[derive(Debug)]
enum WorkspaceMessage {
    Changed,
}

pub struct Workspaces {
    workspaces: Model<Vec<Workspace>>,
}

impl Workspaces {
    pub(crate) fn new(cx: &mut AppContext) -> Self {
        let workspaces_model = cx.new_model(|_| get_workspaces());

        let (tx, mut rx) = mpsc::channel(16);

        let model = workspaces_model.downgrade();

        cx.spawn(|mut cx| async move {
            let mut event_listener = AsyncEventListener::new();
            loop {
                {
                    let tx = tx.clone();

                    event_listener.add_workspace_changed_handler({
                        move |e| {
                            info!("workspace change: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_workspace_added_handler({
                        move |e| {
                            info!("workspace add: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_workspace_deleted_handler({
                        move |e| {
                            info!("workspace delete: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_workspace_moved_handler({
                        move |e| {
                            info!("workspace moved: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_changed_special_handler({
                        move |e| {
                            info!("special workspace changed: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_special_removed_handler({
                        move |e| {
                            info!("special workspace removed: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                {
                    let tx = tx.clone();

                    event_listener.add_active_monitor_changed_handler({
                        move |e| {
                            info!("active monitor changed: {:?}", e);
                            let tx = tx.clone();
                            Box::pin(async move {
                                tx.send(WorkspaceMessage::Changed).await.unwrap();
                            })
                        }
                    });
                }

                event_listener.start_listener_async().await.unwrap();
                cx.background_executor().timer(UPDATE_DEBOUNCE).await;
            }
        })
        .detach();

        cx.spawn(|mut cx| async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    WorkspaceMessage::Changed => {
                        let _ = model.update(&mut cx, |this, cx| {
                            *this = get_workspaces();
                            // cx.refresh();
                        });
                    }
                }
            }
        })
        .detach();

        Self {
            workspaces: workspaces_model,
        }
    }
}

impl Render for Workspaces {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        info!("render workspaces");
        let workspaces = self.workspaces.read(cx);
        h_flex().gap_x_3().justify_between().children(
            workspaces
                .iter()
                .map(|w| {
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
                        .child(w.id.to_string())
                })
                .collect::<Vec<_>>(),
        )
    }
}

impl StatusItemView for Workspaces {}
