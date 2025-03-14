use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, rgb, App, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, RenderOnce, SharedString, Styled, Window,
};

#[derive(IntoElement)]
pub struct Button {
    w: f32,
    h: f32,
    px: f32,
    rounded: f32,
    border: f32,
    bg_color: u32,
    label: SharedString,
    label_color: u32,
    border_color: u32,
    on_click: Arc<dyn Fn(MouseDownEvent, &mut Window, &mut App) + 'static>,
}

impl Button {}

#[allow(dead_code)]
impl Button {
    pub fn new() -> Self {
        Self {
            label: SharedString::from("Button"),
            w: 0.0,
            h: 40.0,
            px: 20.0,
            rounded: 8.0,
            border: 2.0,
            bg_color: 0x45475a,
            label_color: 0xcdd6f4,
            border_color: 0xcba6f7,
            on_click: Arc::new(|_, _, _| println!("Clicked!")),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn size(mut self, w: f32, h: f32) -> Self {
        self.w = w;
        self.h = h;
        self
    }

    pub fn bg(mut self, color: u32) -> Self {
        self.bg_color = color;
        self
    }

    pub fn lable_color(mut self, color: u32) -> Self {
        self.label_color = color;
        self
    }

    pub fn border_color(mut self, color: u32) -> Self {
        self.border_color = color;
        self
    }

    pub fn rounded(mut self, rounded: f32) -> Self {
        self.rounded = rounded;
        self
    }

    pub fn border(mut self, border: f32) -> Self {
        self.border = border;
        self
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(MouseDownEvent, &mut Window, &mut App) + 'static,
    {
        self.on_click = Arc::new(callback);
        self
    }
}

impl Default for Button {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_click = self.on_click.clone();
        div()
            .flex()
            .h(px(self.h))
            .when(self.w != 0.0, |this| this.w(px(self.w)))
            .when(self.w == 0.0, |this| this.w_auto().px(px(self.px)))
            .bg(rgb(self.bg_color))
            .text_color(rgb(self.label_color))
            .border(px(self.border))
            .rounded(px(self.rounded))
            .border_color(rgb(self.border_color))
            .justify_center()
            .content_center()
            .items_center()
            .child(self.label.clone())
            .on_mouse_down(MouseButton::Left, move |event, win, cx| {
                (on_click)(event.clone(), win, cx);
            })
            .into_element()
    }
}
