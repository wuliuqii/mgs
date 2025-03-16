use ui::prelude::*;

pub struct Battery {
    pub level: f32,
}

impl Battery {
    pub fn new<V: 'static>(cx: &mut Context<V>) -> Entity<Self> {
        cx.new(|_cx| Battery { level: 50.0 })
    }
}

impl Render for Battery {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .child(Icon::new(Icons::BatteryCharging))
            .child(self.level.to_string())
    }
}
