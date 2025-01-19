use gpui::{
    px, rems, rgb, AnyView, FontWeight, IntoElement, ParentElement, Render, Styled, View,
    ViewContext, VisualContext,
};

use crate::{
    ui::stack::h_flex,
    widgets::{clock::Clock, workspace::Workspaces},
};

pub trait StatusItemView: Render {}

trait StatusItemViewHandle: Send {
    fn to_any(&self) -> AnyView;
}

pub struct StatusBar {
    left_items: Vec<Box<dyn StatusItemViewHandle>>,
    right_items: Vec<Box<dyn StatusItemViewHandle>>,
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
    fn render_left_tools(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .gap(rems(4.))
            .overflow_x_hidden()
            .children(self.left_items.iter().map(|item| item.to_any()))
    }

    fn render_right_tools(&self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        h_flex()
            .gap(rems(4.))
            .children(self.right_items.iter().map(|item| item.to_any()))
    }
}

impl StatusBar {
    pub fn new(cx: &mut ViewContext<Self>) -> Self {
        let clock = cx.new_view(|cx| Clock::new(cx));
        let workspaces = cx.new_view(|cx| Workspaces::new(cx));
        Self {
            left_items: vec![Box::new(workspaces)],
            right_items: vec![Box::new(clock)],
        }
    }
}

impl<T: StatusItemView> StatusItemViewHandle for View<T> {
    fn to_any(&self) -> AnyView {
        self.clone().into()
    }
}

impl From<&dyn StatusItemViewHandle> for AnyView {
    fn from(val: &dyn StatusItemViewHandle) -> Self {
        val.to_any().clone()
    }
}
