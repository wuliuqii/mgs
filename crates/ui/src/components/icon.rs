use gpui::*;
use prelude::FluentBuilder;

pub enum Icons {
    BatteryLow,
    BatteryMedium,
    BatteryFull,
    BatteryWarning,
    BatteryCharging,
}

impl Icons {
    pub fn path(&self) -> SharedString {
        match self {
            Icons::BatteryLow => "icons/battery-low.svg",
            Icons::BatteryMedium => "icons/battery-medium.svg",
            Icons::BatteryFull => "icons/battery-full.svg",
            Icons::BatteryWarning => "icons/battery-warning.svg",
            Icons::BatteryCharging => "icons/battery-charging.svg",
        }
        .into()
    }
}

#[derive(IntoElement)]
pub struct Icon {
    pub icon: Icons,
    pub size: f32,
    pub color: Rgba,
    pub hover: Option<Rgba>,
}

impl Icon {
    pub fn new(icon: Icons) -> Self {
        Icon {
            icon,
            size: 22.0,
            color: rgb(0x000000),
            hover: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: Rgba) -> Self {
        self.color = color;
        self
    }

    pub fn hover(mut self, color: Rgba) -> Self {
        self.hover = Some(color);
        self
    }
}

impl RenderOnce for Icon {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        svg()
            .flex()
            .items_center()
            .justify_center()
            .size(px(self.size))
            .text_color(self.color)
            .when_some(self.hover, |this, color| {
                this.hover(|this| this.text_color(color))
            })
            .path(self.icon.path())
    }
}
