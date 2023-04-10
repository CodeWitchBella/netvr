use tokio::runtime::Handle;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = Handle::current();
    let responder = libmdns::Responder::spawn(&handle)?;
    let _svc = responder.register(
        "_netvr._udp".to_owned(),
        "NetVR Server".to_owned(),
        12345,
        &["path=/"],
    );

    loop {
        ::std::thread::sleep(::std::time::Duration::from_secs(10));
    }
}
