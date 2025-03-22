use futures_util::StreamExt;
use gpui::{AsyncApp, WeakEntity};
use services::upower::{self, BatteryState};
use ui::prelude::*;

const fn u32_to_battery_state(number: u32) -> Result<BatteryState, u32> {
    if number == (BatteryState::Unknown as u32) {
        Ok(BatteryState::Unknown)
    } else if number == (BatteryState::Charging as u32) {
        Ok(BatteryState::Charging)
    } else if number == (BatteryState::Discharging as u32) {
        Ok(BatteryState::Discharging)
    } else if number == (BatteryState::Empty as u32) {
        Ok(BatteryState::Empty)
    } else if number == (BatteryState::FullyCharged as u32) {
        Ok(BatteryState::FullyCharged)
    } else if number == (BatteryState::PendingCharge as u32) {
        Ok(BatteryState::PendingCharge)
    } else if number == (BatteryState::PendingDischarge as u32) {
        Ok(BatteryState::PendingDischarge)
    } else {
        Err(number)
    }
}

#[derive(Debug, Clone)]
struct Properties {
    percentage: f64,
    state: BatteryState,
    time_to_full: i64,
    time_to_empty: i64,
}

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
                let proxy = upower::create_upower_proxy().await.unwrap();

                let device_interface_name =
                    zbus::names::InterfaceName::from_static_str("org.freedesktop.UPower.Device")
                        .expect("failed to create zbus InterfaceName");
                let properties = proxy.get_all(device_interface_name.clone()).await.unwrap();
                let percentage = properties["Percentage"]
                    .downcast_ref::<f64>()
                    .expect("expected percentage: f64 in HashMap of all properties");
                let state = u32_to_battery_state(
                    properties["State"]
                        .downcast_ref::<u32>()
                        .expect("expected State: u32 in HashMap of all properties"),
                )
                .unwrap_or(BatteryState::Unknown);
                let time_to_full = properties["TimeToFull"]
                    .downcast_ref::<i64>()
                    .expect("expected TimeToFull: i64 in HashMap of all properties");
                let time_to_empty = properties["TimeToEmpty"]
                    .downcast_ref::<i64>()
                    .expect("expected TimeToEmpty: i64 in HashMap of all properties");

                let mut properties = Properties {
                    percentage,
                    state,
                    time_to_full,
                    time_to_empty,
                };
                this.update(cx, |this: &mut Self, cx| {
                    this.update(&properties);
                    cx.notify();
                })
                .ok();

                let mut prop_changed_stream = proxy.receive_properties_changed().await.unwrap();
                while let Some(signal) = prop_changed_stream.next().await {
                    let args = signal.args().expect("Invalid signal arguments");
                    if args.interface_name != device_interface_name {
                        continue;
                    }

                    for (name, changed_value) in args.changed_properties {
                        match name {
                            "Percentage" => {
                                properties.percentage = changed_value
                                    .downcast::<f64>()
                                    .expect("expected Percentage to be f64");
                            }
                            "State" => {
                                properties.state = u32_to_battery_state(
                                    changed_value.downcast::<u32>().unwrap_or(0),
                                )
                                .expect("expected State to be BatteryState");
                            }
                            "TimeToFull" => {
                                properties.time_to_full = changed_value
                                    .downcast::<i64>()
                                    .expect("expected TimeToFull to be i64");
                            }
                            "TimeToEmpty" => {
                                properties.time_to_empty = changed_value
                                    .downcast::<i64>()
                                    .expect("expected TimeToEmpty to be i64");
                            }
                            _ => {}
                        }
                    }

                    this.update(cx, |this: &mut Self, cx| {
                        this.update(&properties);
                        cx.notify();
                    })
                    .ok();
                }
            })
            .detach();

            upower
        })
    }

    fn update(&mut self, properties: &Properties) {
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
