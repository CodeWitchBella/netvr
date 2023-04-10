use trust_dns_resolver::{config::*, dns_sd::DnsSdHandle, Name, TokioAsyncResolver};

const SERVICE_NAME: &'static str = "_spotify-connect._tcp.local.";

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello there! I'm looking for {SERVICE_NAME}");

    // Construct a new Resolver with default configuration options
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default())?;

    // Lookup the IP addresses associated with a name.
    // This returns a future that will lookup the IP addresses, it must be run in the Core to
    //  to get the actual result.
    let response = resolver
        .list_services(Name::from_utf8(SERVICE_NAME)?)
        .await?;

    // There can be many addresses associated with the name
    for service in response.iter() {
        println!("service: {:?}", service);
    }

    Ok(())
}
