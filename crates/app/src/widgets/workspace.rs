use futures::StreamExt;
use futures_signals::signal::SignalExt;

use services::hyprland::{self, Workspace, change_workspace_to};

use ui::prelude::*;

pub struct Workspaces {
    workspaces: Vec<Workspace>,
    active: usize,
}

impl Workspaces {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let workspaces = Self {
                workspaces: Vec::new(),
                active: 0,
            };

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let subscriber = hyprland::Subscriber::new().await.unwrap();

                cx.background_spawn({
                    let subscriber = subscriber.clone();
                    async move {
                        subscriber.run().unwrap();
                    }
                })
                .detach();

                let mut signal = subscriber.subscribe().to_stream();
                while let Some(data) = signal.next().await {
                    this.update(cx, |this: &mut Self, cx| {
                        this.update(&data);
                        cx.notify();
                    })
                    .ok();
                }
            })
            .detach();

            workspaces
        })
    }

    fn update(&mut self, data: &hyprland::WorkspacesData) {
        self.active = data.active;
        self.workspaces = data.workspaces.clone();
    }
}

fn workspace_button(w: Workspace) -> impl IntoElement {
    Button::new()
        .label(w.name)
        .size(28.)
        .border(2.)
        .rounded(10.)
        .when(w.active, |this| this.border_color(rgb(0xffc0cb)))
        .on_click(move |_, _, _| {
            change_workspace_to(w.id).ok();
        })
}

impl Render for Workspaces {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        let workspaces = self.workspaces.clone();
        h_flex().justify_between().children(
            workspaces
                .into_iter()
                .filter(|w| w.visible)
                .map(|w| workspace_button(w)),
        )
    }
}
