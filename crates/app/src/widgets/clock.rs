use std::time::Duration;

use chrono::{DateTime, Local};
use ui::prelude::*;

pub struct Clock {
    date: DateTime<Local>,
}

impl Clock {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
            let clock = Self { date: Local::now() };

            cx.spawn(async move |this, cx| loop {
                this.update(cx, |this: &mut Clock, cx| {
                    this.date = Local::now();
                    cx.notify();
                })
                .ok();

                Timer::after(Duration::from_secs(1)).await;
            })
            .detach();

            clock
        })
    }
}

impl Render for Clock {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().child(self.date.format("%H:%M").to_string())
    }
}
