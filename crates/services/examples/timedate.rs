use services::timedate::create_timedate_proxy;

#[tokio::main]
pub async fn main() {
    let proxy = create_timedate_proxy().await.unwrap();

    let device_interface_name =
        zbus::names::InterfaceName::from_static_str("org.freedesktop.timedate1")
            .expect("failed to create zbus InterfaceName");

    let properties = proxy.get_all(device_interface_name).await.unwrap();
    for (name, value) in properties {
        println!("Property: {} = {:?}", name, value);
    }
}
