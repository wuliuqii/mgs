use futures_signals::signal::SignalExt;
use futures_util::StreamExt;
use gpui::{AsyncApp, WeakEntity};
use services::upower::{self, BatteryState, UpowerData};
use ui::prelude::*;

pub struct Upower {
    label: String,
    status: BatteryState,
    icon_path: SharedString,
    icon_color: Option<Rgba>,
}

impl Upower {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let upower = Upower {
                label: "Battery".to_string(),
                status: BatteryState::Unknown,
                icon_path: SharedString::new("icons/battery.svg"),
                icon_color: None,
            };

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let subscriber = upower::Subscriber::new().await.unwrap();

                cx.background_spawn({
                    let subscriber = subscriber.clone();
                    async move {
                        subscriber.run().await.unwrap();
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

            upower
        })
    }

    fn update(&mut self, properties: &UpowerData) {
        self.label = format!("{}", properties.percentage);
        self.status = properties.state;
        self.icon_path = match properties.state {
            BatteryState::FullyCharged | BatteryState::Charging | BatteryState::PendingCharge => {
                "icons/battery-charging.svg"
            }
            BatteryState::Discharging | BatteryState::PendingDischarge => {
                match properties.percentage {
                    p if p <= 10.0 => {
                        self.icon_color = Some(rgb(0xff8000));
                        "icons/battery-warning.svg"
                    }
                    p if p <= 30.0 => {
                        self.icon_color = Some(rgb(0xff0000));
                        "icons/battery-low.svg"
                    }
                    p if p <= 80.0 => "icons/battery-medium.svg",
                    _ => "icons/battery-full.svg",
                }
            }
            _ => "icons/battery.svg",
        }
        .into();
    }
}

impl Render for Upower {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .child(
                Icon::new(self.icon_path.clone())
                    .when_some(self.icon_color, |this, color| this.color(color)),
            )
            .child(self.label.to_string())
    }
}
