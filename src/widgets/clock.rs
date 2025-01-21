use std::time::Duration;

use chrono::{DateTime, Local};
use gpui::{div, rgb, ParentElement, Render, Styled, View, ViewContext, VisualContext};
use tracing::debug;

const UPDATE_DEBOUNCE: Duration = Duration::from_millis(1000);

pub struct Clock {
    date: DateTime<Local>,
}

impl Clock {
    pub fn new<V: 'static>(cx: &mut ViewContext<V>) -> View<Self> {
        cx.new_view(|cx| {
            let clock = Self { date: Local::now() };

            cx.spawn(|this, mut cx| async move {
                loop {
                    this.update(&mut cx, |this: &mut Clock, cx| {
                        this.date = Local::now();
                        cx.notify();
                    })
                    .ok();

                    cx.background_executor().timer(UPDATE_DEBOUNCE).await;
                }
            })
            .detach();

            clock
        })
    }
}

impl Render for Clock {
    fn render(&mut self, _cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        debug!("render clock");

        div()
            .flex()
            .text_color(rgb(0x4c4f69))
            .child(self.date.format("%H:%M:%S").to_string())
    }
}
