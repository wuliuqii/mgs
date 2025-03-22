use std::sync::Arc;

use gpui::{
    div, prelude::FluentBuilder, px, rgb, App, InteractiveElement, IntoElement, MouseButton,
    MouseDownEvent, ParentElement, RenderOnce, Rgba, SharedString, Styled, Window,
};

#[derive(IntoElement)]
pub struct Button {
    size: Option<f32>,
    rounded: f32,
    border: f32,
    bg_color: Option<Rgba>,
    hover_bg: Option<Rgba>,
    label: SharedString,
    border_color: Option<Rgba>,
    on_click: Arc<dyn Fn(MouseDownEvent, &mut Window, &mut App) + 'static>,
}

impl Button {}

#[allow(dead_code)]
impl Button {
    pub fn new() -> Self {
        Self {
            label: SharedString::from("Button"),
            size: None,
            rounded: 4.0,
            border: 0.0,
            bg_color: None,
            border_color: None,
            hover_bg: Some(rgb(0xe1dede)),
            on_click: Arc::new(|_, _, _| println!("Clicked!")),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = Some(size);
        self
    }

    pub fn bg(mut self, color: Rgba) -> Self {
        self.bg_color = Some(color);
        self
    }

    pub fn border_color(mut self, color: Rgba) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn hover(mut self, color: Rgba) -> Self {
        self.hover_bg = Some(color);
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
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let on_click = self.on_click.clone();
        div()
            .flex()
            .border(px(self.border))
            .rounded(px(self.rounded))
            .child(self.label.clone())
            .when_some(self.size, |this, size| this.size(px(size)).px(px(5.)))
            .when_none(&self.size, |this| this.w_auto())
            .when_some(self.hover_bg, |this, color| {
                this.hover(|this| this.bg(color))
            })
            .when_some(self.border_color, |this, color| this.border_color(color))
            .on_mouse_down(MouseButton::Left, move |event, win, cx| {
                (on_click)(event.clone(), win, cx);
            })
            .justify_center()
            .content_center()
            .items_center()
            .into_element()
    }
}
