use mdns::Error;
use simple_mdns::async_discovery::ServiceDiscovery;
use std::{net::SocketAddr, str::FromStr};

const SERVICE_NAME: &'static str = "_netvr._udp.local";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut discovery =
        ServiceDiscovery::new("_my_inst", SERVICE_NAME, 15).expect("Invalid Service Name");
    let _ = discovery
        .add_service_info(SocketAddr::from_str("192.168.1.22:8090").unwrap().into())
        .await;
    loop {
        let known = discovery.get_known_services().await;
        for service in known {
            println!("Found service: {:?}", service);
            let addrs = service
                .get_socket_addresses()
                .map(|addr| addr.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            println!("    {:?}", addrs);
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
}
