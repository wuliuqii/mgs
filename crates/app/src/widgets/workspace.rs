use hyprland::{
    event_listener::{
        ChangedSpecialEventData, EventListener, MonitorEventData, WindowMoveEvent,
        WorkspaceEventData, WorkspaceMovedEventData,
    },
    shared::{Address, HyprData, HyprDataActive, HyprDataVec},
};
use tracing::debug;

use ui::prelude::*;

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
                    // name: w
                    //     .name
                    //     .split(":")
                    //     .last()
                    //     .map_or_else(|| "".to_string(), |s| s.to_owned()),
                    name: "S".to_string(),
                    monitor_id: w.monitor_id.map(|id| id as usize),
                    active: monitors.iter().any(|m| m.special_workspace.id == w.id),
                    windows: w.windows,
                }]
            } else {
                vec![Workspace {
                    id: w.id,
                    name: w.name.clone(),
                    monitor_id: w.monitor_id.map(|id| id as usize),
                    active: Some(w.id) == active.as_ref().map(|w| w.id),
                    windows: w.windows,
                }]
            }
        })
        .collect()
}

pub struct Workspaces {
    workspaces: Vec<Workspace>,
}

impl Workspaces {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let instance = Self {
                workspaces: get_workspaces(),
            };

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let mut event_listener = EventListener::new();

                let entity = this.upgrade().unwrap();

                // TODO: maybe can be more elegant
                // Define a macro to create event handlers for different event types
                macro_rules! create_event_handler {
                    ($cx:expr, $entity:expr, $event_type:ty) => {{
                        let entity = $entity.clone();
                        let cx = $cx.clone();
                        move |e: $event_type| {
                            debug!("Event received: {:?}", e);
                            let mut cx_clone = cx.clone();
                            entity
                                .update(&mut cx_clone, |this, cx| {
                                    this.workspaces = get_workspaces();
                                    cx.notify();
                                })
                                .ok();
                        }
                    }};
                }

                // Register event handlers with appropriate types
                event_listener.add_workspace_changed_handler(create_event_handler!(
                    cx,
                    entity,
                    WorkspaceEventData
                ));
                event_listener.add_workspace_added_handler(create_event_handler!(
                    cx,
                    entity,
                    WorkspaceEventData
                ));
                event_listener.add_workspace_moved_handler(create_event_handler!(
                    cx,
                    entity,
                    WorkspaceMovedEventData
                ));
                event_listener.add_workspace_deleted_handler(create_event_handler!(
                    cx,
                    entity,
                    WorkspaceEventData
                ));
                event_listener.add_active_monitor_changed_handler(create_event_handler!(
                    cx,
                    entity,
                    MonitorEventData
                ));
                event_listener.add_changed_special_handler(create_event_handler!(
                    cx,
                    entity,
                    ChangedSpecialEventData
                ));
                event_listener
                    .add_special_removed_handler(create_event_handler!(cx, entity, String));
                event_listener
                    .add_window_closed_handler(create_event_handler!(cx, entity, Address));
                event_listener.add_window_moved_handler(create_event_handler!(
                    cx,
                    entity,
                    WindowMoveEvent
                ));
                event_listener.start_listener_async().await.unwrap();
            })
            .detach();

            instance
        })
    }
}

impl Render for Workspaces {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let workspaces = self.workspaces.clone();
        h_flex()
            .justify_between()
            .children(workspaces.into_iter().map(|w| {
                Button::new()
                    .label(w.name.to_string())
                    .size(28.)
                    .border(2.)
                    .rounded(10.)
                    .when(w.active, |this| this.border_color(rgb(0xffc0cb)))
                    .on_click(move |_, _, _| {
                        hyprland::dispatch::Dispatch::call(
                            hyprland::dispatch::DispatchType::Workspace(
                                hyprland::dispatch::WorkspaceIdentifierWithSpecial::Id(w.id),
                            ),
                        )
                        .ok();
                    })
            }))
    }
}
