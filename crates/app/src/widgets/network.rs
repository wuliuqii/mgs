use services::network;

use futures_signals::signal::SignalExt;
use futures_util::StreamExt;

use services::network::ActiveConnectionInfo::WiFi;
use tracing::{debug, error};
use ui::prelude::*;

#[allow(dead_code)]
#[derive(PartialEq)]
enum State {
    WiredConnected,
    WifiConnected,
    VpnConnected,
    WifiDisconnected,
    Offline,
}

pub struct NetworkWidget {
    state: State,
    icon_path: SharedString,
    rx_speed: String,
    tx_speed: String,
}

impl NetworkWidget {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let widget = Self {
                state: State::WifiConnected,
                icon_path: SharedString::new("icons/wifi-off.svg"),
                rx_speed: "0.0 kb/s".to_string(),
                tx_speed: "0.0 kb/s".to_string(),
            };

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let mut client = network::Client::new().await.unwrap();
                // todo: move to config
                client.set_refresh_rate_ms(5000);
                cx.background_spawn({
                    let client = client.clone();
                    async move {
                        client.run().await.unwrap();
                    }
                })
                .detach();

                let mut signal = client.subscribe().to_stream();
                while let Some(data) = signal.next().await {
                    this.update(cx, |this: &mut Self, cx| {
                        this.update(data);
                        cx.notify();
                    })
                    .ok();
                }
            })
            .detach();

            widget
        })
    }

    fn update(&mut self, data: network::NetworkData) {
        debug!("Network data updated: {:?}", data);

        if !data.wifi_enabled {
            self.state = State::Offline;
            // todo: need more icons
            self.icon_path = SharedString::new("icons/wifi-off.svg");
            return;
        }

        if data.active_connections.is_empty() {
            self.state = State::WifiDisconnected;
            self.icon_path = SharedString::new("icons/wifi-off.svg");
            return;
        }

        let active = data.active_connections.first().unwrap();
        match active {
            WiFi {
                strength, device, ..
            } => {
                match strength {
                    s if *s < 25 => {
                        self.icon_path = SharedString::new("icons/wifi-low.svg");
                    }
                    s if *s < 50 => {
                        self.icon_path = SharedString::new("icons/wifi-high.svg");
                    }
                    _ => {
                        self.icon_path = SharedString::new("icons/wifi-full.svg");
                    }
                }
                self.state = State::WifiConnected;
                data.network_statistics.iter().for_each(|stat| {
                    if stat.device == *device {
                        self.rx_speed = format_speed(stat.rx_speed());
                        self.tx_speed = format_speed(stat.tx_speed());
                    }
                });
            }
            _ => {
                error!("unimplemented");
            }
        }
    }
}

fn format_speed(speed: f64) -> String {
    if speed < 1000000.0 {
        format!("{:.2} kb/s", speed / 1000.0)
    } else {
        format!("{:.2} mb/s", speed / 1000000.0)
    }
}

impl Render for NetworkWidget {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<'_, Self>) -> impl IntoElement {
        h_flex()
            .when(self.state == State::WifiConnected, |elm| {
                elm.text_size(px(9.)).child(
                    div()
                        .v_flex()
                        .child(self.tx_speed.to_string())
                        .child(self.rx_speed.to_string()),
                )
            })
            .child(Icon::new(self.icon_path.clone()).size(18.))
    }
}
