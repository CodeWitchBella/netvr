use futures_util::{pin_mut, stream::StreamExt};
use mdns::{Error, Record, RecordKind};
use std::{net::IpAddr, time::Duration};

const SERVICE_NAME: &'static str = "_netvr._udp.local";

#[tokio::main]
async fn main() -> Result<(), Error> {
    println!("Hello there! I'm looking for NetVR devices...");
    // Iterate through responses from each Cast device, asking for new devices every 15s
    let stream = mdns::discover::all(SERVICE_NAME, Duration::from_secs(5))?.listen();
    pin_mut!(stream);

    while let Some(Ok(response)) = stream.next().await {
        let addr = response.records().filter_map(self::to_ip_addr).next();

        if let Some(addr) = addr {
            println!("found cast device at {}", addr);
        } else {
            println!("cast device does not advertise address");
        }
    }

    Ok(())
}

fn to_ip_addr(record: &Record) -> Option<IpAddr> {
    match record.kind {
        RecordKind::A(addr) => Some(addr.into()),
        RecordKind::AAAA(addr) => Some(addr.into()),
        _ => None,
    }
}
