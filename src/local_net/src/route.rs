use futures::TryStreamExt;
use netlink_packet_route::RouteMessage;
use rtnetlink::Error::RequestFailed;
use rtnetlink::{Handle, IpVersion};
use std::net::Ipv4Addr;

pub async fn get_routes(handle: &Handle) -> Result<Vec<RouteMessage>, Box<dyn std::error::Error>> {
    let mut routes = handle.route().get(IpVersion::V4).execute();
    let mut route_message_vec = Vec::new();

    while let Some(route_message) = routes.try_next().await? {
        route_message_vec.push(route_message);
    }

    Ok(route_message_vec)
}

pub async fn set_default_route(
    handle: &Handle,
    interface: u32,
    gateway: Ipv4Addr,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = handle
        // Base route request:
        .route()
        .add()
        .v4()
        .replace()
        // Gateway and interface:
        .output_interface(interface)
        .gateway(gateway)
        // Kernel address:
        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0);

    request.execute().await?;

    Ok(())
}

pub async fn flush_routes(handle: &Handle) -> Result<(), Box<dyn std::error::Error>> {
    let mut routes = handle.route().get(IpVersion::V4).execute();

    while let Some(route_message) = routes.try_next().await? {
        let handle = handle.clone();
        handle.route().del(route_message).execute().await?;
    }

    // Verify and return:
    match get_routes(handle).await?.len() {
        0 => Ok(()),
        _ => Err(Box::new(RequestFailed)),
    }
}
