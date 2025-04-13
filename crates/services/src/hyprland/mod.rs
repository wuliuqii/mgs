use anyhow::Ok;
use futures_signals::signal::{Mutable, MutableSignalCloned};
use hyprland::dispatch::{Dispatch, DispatchType, WorkspaceIdentifierWithSpecial};
use hyprland::event_listener::EventListener;
use hyprland::shared::{HyprData, HyprDataActive};

#[derive(Debug, Clone)]
pub struct Workspace {
    pub id: i32,
    pub name: String,
    // pub monitor_id: Option<usize>,
    pub active: bool,
    pub visible: bool,
    // pub windows: u16,
}

#[derive(Debug, Clone)]
pub struct WorkspacesData {
    pub workspaces: Vec<Workspace>,
    pub active: usize,
}

impl WorkspacesData {
    fn init() -> anyhow::Result<Self> {
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

        Ok(Self { workspaces, active })
    }
}

#[derive(Debug, Clone)]
pub struct Subscriber {
    data: Mutable<WorkspacesData>,
}

impl Subscriber {
    pub async fn new() -> anyhow::Result<Self> {
        Ok(Self {
            data: Mutable::new(WorkspacesData::init()?),
        })
    }

    pub fn run(&self) -> anyhow::Result<()> {
        let mut event_listener = EventListener::new();

        {
            let data = self.data.clone();
            event_listener.add_workspace_changed_handler(move |evt| {
                let mut data = data.lock_mut();
                let prev_active = data.active;
                data.workspaces[prev_active].active = false;
                let active = evt.id as usize;
                data.workspaces[active].active = true;
                data.workspaces[active].visible = true;
                data.active = active;
            });
        }

        {
            let data = self.data.clone();
            event_listener.add_workspace_added_handler(move |evt| {
                let id = if evt.id < 0 { 0 } else { evt.id as usize };
                let mut data = data.lock_mut();
                data.workspaces[id].visible = true;
            });
        }

        {
            let data = self.data.clone();
            event_listener.add_workspace_deleted_handler(move |evt| {
                let id = if evt.id < 0 { 0 } else { evt.id as usize };
                let mut data = data.lock_mut();
                data.workspaces[id].visible = false;
            });
        }

        event_listener.start_listener()?;

        Ok(())
    }

    pub fn subscribe(&self) -> MutableSignalCloned<WorkspacesData> {
        self.data.signal_cloned()
    }
}

pub fn change_workspace_to(id: i32) -> anyhow::Result<()> {
    Dispatch::call(DispatchType::Workspace(WorkspaceIdentifierWithSpecial::Id(
        id,
    )))?;

    Ok(())
}
