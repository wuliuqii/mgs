use services::audio::{self, Client};
use std::sync::Arc;
use tracing::{debug, info};
use ui::prelude::*;

pub struct Volume {
    slider: Entity<Slider>,
    muted: bool,
    client: Arc<Client>,
    volume: f32,
    sink_name: String,
}

impl Volume {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let vol_slider = cx.new(|_| {
                Slider::new()
                    .min(0.0)
                    .max(100.)
                    .step(1.0)
                    .default_value(50.)
            });

            let client = audio::create_client();

            let this = Self {
                slider: vol_slider,
                muted: false,
                client,
                volume: 0.5,
                sink_name: "default".to_string(),
            };

            // 订阅 slider 变化，设置系统音量
            cx.subscribe(
                &this.slider,
                |state: &mut Self,
                 _slider: Entity<Slider>,
                 event: &SliderEvent,
                 _cx: &mut Context<Self>| match event {
                    SliderEvent::Change(value) => {
                        debug!("Volume changed to {}", value);

                        state.client.set_sink_volume(&state.sink_name, *value);
                    }
                },
            )
            .detach();

            cx.spawn(async move |this: WeakEntity<Self>, cx: &mut AsyncApp| {
                let client = audio::create_client();

                let mut signal = client.subscribe();
                while let Ok(event) = signal.recv().await {
                    info!("Received audio signal: {:?}", event);

                    if let audio::Event::UpdateSink(sink) = event {
                        if let Some(this) = this.upgrade() {
                            cx.update_entity(&this, |state, cx| {
                                state.volume = sink.volume;
                                state.muted = sink.muted;
                                state.sink_name = sink.index.to_string();
                                cx.update_entity(&state.slider, |slider, cx| {
                                    slider.set_value(sink.volume, cx);
                                });
                            })
                            .ok();
                        }
                    }
                }
            })
            .detach();

            this
        })
    }

    fn toggle_mute(
        &mut self,
        _event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.muted = !self.muted;
        self.client.set_sink_muted(&self.sink_name, self.muted);
        cx.notify();
    }
}

impl Render for Volume {
    fn render(&mut self, _window: &mut ui::Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let value = self.volume;
        let icon_path = if self.muted {
            "icons/volume-mute.svg"
        } else if value < 30. {
            "icons/volume-low.svg"
        } else if value < 70. {
            "icons/volume-medium.svg"
        } else {
            "icons/volume-high.svg"
        };

        h_flex()
            .gap_1()
            .child(
                div()
                    .child(Icon::new(icon_path.into()).size(18.))
                    .on_mouse_down(MouseButton::Left, cx.listener(Self::toggle_mute)),
            )
            // todo: only show slider on hover
            .child(div().child(self.slider.clone()).w_20())
            .child(
                div()
                    .w_7()
                    .text_center()
                    .child(format!("{:00.0}", value))
                    .overflow_hidden(),
            )
    }
}
