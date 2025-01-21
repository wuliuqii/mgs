use gpui::{
    point, px, size, Anchor, App, AppContext, Bounds, KeyboardInteractivity, Layer,
    LayerShellSettings, VisualContext, WindowBounds, WindowKind, WindowOptions,
};
use status_bar::StatusBar;

mod status_bar;
mod widgets;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    App::new().run(|cx: &mut AppContext| {
        let height = px(30.0);
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
            |cx| cx.new_view(StatusBar::new),
        )
        .unwrap();
    });
}
