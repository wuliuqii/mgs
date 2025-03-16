use assets::Assets;
use status_bar::StatusBar;
use ui::{
    point, px, size, Anchor, App, AppContext, Application, Bounds, KeyboardInteractivity, Layer,
    LayerShellSettings, WindowBounds, WindowKind, WindowOptions,
};

mod status_bar;
mod widgets;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let assets = Assets {};
    let app = Application::new().with_assets(assets);

    app.run(|cx: &mut App| {
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

        let opts = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            kind: WindowKind::LayerShell(layer_shell_settings),
            ..Default::default()
        };

        cx.open_window(opts, |_, cx| cx.new(StatusBar::new))
            .unwrap();
    });
}
