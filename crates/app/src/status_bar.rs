use gpui::{
    px, rems, rgb, AnyView, FontWeight, IntoElement, ParentElement, Render, Styled, ViewContext,
};
use ui::h_flex;

use crate::widgets::{clock::Clock, workspace::Workspaces};

pub struct StatusBar {
    left_items: Vec<AnyView>,
    right_items: Vec<AnyView>,
}

impl Render for StatusBar {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .w_full()
            .h_full()
            .justify_between()
            .gap(rems(8.))
            .px(px(4.))
            .bg(rgb(0xeff1f5))
            .font_family("MonoLisa")
            .font_weight(FontWeight::EXTRA_BOLD)
            .text_sm()
            .child(self.render_left_tools(cx))
            .child(self.render_right_tools(cx))
    }
}

impl StatusBar {
    fn render_left_tools(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .gap(rems(4.))
            .overflow_x_hidden()
            .children(self.left_items.iter().cloned())
    }

    fn render_right_tools(&self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .gap(rems(4.))
            .children(self.right_items.iter().cloned())
    }
}

impl StatusBar {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let clock = Clock::new(cx).clone();
        let workspaces = Workspaces::new(cx).clone();
        Self {
            left_items: vec![workspaces.into()],
            right_items: vec![clock.into()],
        }
    }
}
