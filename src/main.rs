use gpui::{
    div, point, prelude::*, px, rgb, size, Anchor, App, AppContext, Bounds, KeyboardInteractivity,
    Layer, LayerShellSettings, SharedString, ViewContext, WindowBounds, WindowKind, WindowOptions,
};

struct HelloWorld {
    text: SharedString,
}

impl Render for HelloWorld {
    fn render(&mut self, _cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_row()
            .items_center()
            .w_full()
            .justify_between()
            .gap_3()
            .py_3()
            .px_3()
            .bg(rgb(0x505050))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .overflow_x_hidden()
                    .border_1()
                    .border_color(rgb(0x0000ff))
                    .text_color(rgb(0xffffff))
                    .child(format!("hello, {}!", &self.text)),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(div().size_8().bg(gpui::red()))
                    .child(div().size_8().bg(gpui::green()))
                    .child(div().size_8().bg(gpui::blue()))
                    .child(div().size_8().bg(gpui::yellow()))
                    .child(div().size_8().bg(gpui::black()))
                    .child(div().size_8().bg(gpui::white())),
            )
    }
}

fn main() {
    App::new().run(|cx: &mut AppContext| {
        let height = px(50.0);
        let bounds = Bounds {
            origin: point(px(0.), px(0.)),
            size: size(px(1440.), height),
        };
        let layer_shell_settings = LayerShellSettings {
            layer: Layer::Top,
            anchor: Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            exclusive_zone: Some(height),
            keyboard_interactivity: KeyboardInteractivity::None,
            namespace: "simple bar".to_string(),
            ..Default::default()
        };
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                kind: WindowKind::LayerShell(layer_shell_settings),
                ..Default::default()
            },
            |cx| {
                cx.new_view(|_cx| HelloWorld {
                    text: "World".into(),
                })
            },
        )
        .unwrap();
    });
}
