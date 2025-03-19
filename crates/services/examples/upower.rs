use futures_util::stream::StreamExt;
use services::upower::create_upower_proxy;

#[tokio::main]
async fn main() {
    let display_proxy = create_upower_proxy().await.unwrap();

    let device_interface_name =
        zbus::names::InterfaceName::from_static_str("org.freedesktop.UPower.Device")
            .expect("failed to create zbus InterfaceName");

    let mut prop_changed_stream = display_proxy.receive_properties_changed().await.unwrap();
    while let Some(signal) = prop_changed_stream.next().await {
        let args = signal.args().expect("Invalid signal arguments");
        if args.interface_name != device_interface_name {
            continue;
        }

        for (name, changed_value) in args.changed_properties {
            match name {
                "Percentage" => println!(
                    "Percentage: {}",
                    changed_value
                        .downcast_ref::<f64>()
                        .expect("expected Percentage to be f64")
                ),
                "IconName" => println!(
                    "IconName: {:?}",
                    changed_value
                        .downcast_ref::<&str>()
                        .expect("expected IconName to be &str")
                ),
                "State" => println!(
                    "State: {:?}",
                    changed_value
                        .downcast_ref::<u32>()
                        .expect("expected State to be u32")
                ),
                "TimeToFull" => println!(
                    "TimeToFull: {:?}",
                    changed_value
                        .downcast_ref::<i64>()
                        .expect("expected TimeToFull to be u32")
                ),
                "TimeToEmpty" => println!(
                    "TimeToEmpty: {:?}",
                    changed_value
                        .downcast_ref::<i64>()
                        .expect("expected TimeToEmpty to be u32")
                ),
                _ => {}
            }
        }
    }
}
