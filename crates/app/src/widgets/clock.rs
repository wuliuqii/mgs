use std::time::Duration;

use chrono::{DateTime, Local};
use ui::prelude::*;

const UPDATE_DEBOUNCE: Duration = Duration::from_millis(1000);

pub struct Clock {
    date: DateTime<Local>,
}

impl Clock {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|cx| {
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
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().flex().child(self.date.format("%H:%M").to_string())
    }
}
