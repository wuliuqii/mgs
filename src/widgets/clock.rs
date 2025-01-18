use std::time::Duration;

use chrono::{DateTime, Local};
use gpui::{div, rgb, AppContext, Context, Model, ParentElement, Render, Styled};

use crate::status_bar::StatusItemView;

const UPDATE_DEBOUNCE: Duration = Duration::from_millis(1000);

pub struct Clock {
    date: Model<DateTime<Local>>,
}

impl Clock {
    pub fn new(cx: &mut AppContext) -> Self {
        let clock_model = cx.new_model(|_| Local::now());

        let model = clock_model.downgrade();
        cx.spawn(|mut cx| async move {
            loop {
                model
                    .update(&mut cx, |this, cx| {
                        *this = Local::now();
                        cx.refresh();
                    })
                    .ok();

                cx.background_executor().timer(UPDATE_DEBOUNCE).await;
            }
        })
        .detach();

        Self { date: clock_model }
    }
}

impl Render for Clock {
    fn render(&mut self, cx: &mut gpui::ViewContext<Self>) -> impl gpui::IntoElement {
        let date = self.date.read(cx);
        div()
            .flex()
            .border_1()
            .border_color(rgb(0x0000ff))
            .text_color(rgb(0xfffffff))
            .text_lg()
            .child(date.format("%H:%M:%S").to_string())
    }
}

impl StatusItemView for Clock {}
