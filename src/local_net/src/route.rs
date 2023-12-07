use futures::{Future, TryStreamExt};
use log::{debug, error, info, trace, warn};
use netlink_packet_route::route::Nla;
use netlink_packet_route::RouteMessage;
use rtnetlink::{Handle, IpVersion};
use std::net::{Ipv4Addr, Ipv6Addr};

use crate::RTNetlinkError;

/// Get all routes from the kernel.
pub async fn get_routes(handle: &Handle) -> Result<Vec<RouteMessage>, RTNetlinkError> {
    let mut routes = handle.route().get(IpVersion::V4).execute();
    let mut route_message_vec = Vec::new();

    while let Some(route_message) = routes.try_next().await.map_err(RTNetlinkError::RTNetlink)? {
        route_message_vec.push(route_message);
    }

    Ok(route_message_vec)
}

/// Set the default route for an interface.
pub async fn set_default_route(
    handle: &Handle,
    iface_idx: u32,
    gateway: Ipv4Addr,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = handle
        // Base route request:
        .route()
        .add()
        .v4()
        .replace()
        // Gateway and interface:
        .output_interface(iface_idx)
        .gateway(gateway)
        // Kernel address:
        .destination_prefix(Ipv4Addr::new(0, 0, 0, 0), 0);

    request.execute().await?;

    Ok(())
}

pub async fn modify_routes<F, Fut>(
    handle: &Handle,
    ip_version: IpVersion,
    function: F,
) -> Result<(), RTNetlinkError>
where
    F: FnOnce(RouteMessage) -> Fut + Copy,
    Fut: Future<Output = Result<(), RTNetlinkError>>,
{
    let mut routes = handle.route().get(ip_version).execute();

    while let Some(route_message) = routes.try_next().await.map_err(RTNetlinkError::RTNetlink)? {
        function(route_message.clone()).await?;
    }

    Ok(())
}

/// Flush all routes from an interface.
pub async fn flush_routes(
    handle: &Handle,
    ip_version: IpVersion,
    iface_idx: u32,
) -> Result<(), RTNetlinkError> {
    modify_routes(
        handle,
        ip_version.clone(),
        async move |route_message| -> Result<(), RTNetlinkError> {
            for nla in route_message.nlas.iter() {
                if let Nla::Oif(oif) = nla {
                    if *oif == iface_idx {
                        let request = handle.route().del(route_message.clone());
                        request.execute().await.map_err(RTNetlinkError::RTNetlink)?;
                    }
                }
            }
            Ok(())
        },
    )
    .await?;

    let mut routes = handle.route().get(ip_version).execute();

    while let Some(route_message) = routes.try_next().await.map_err(RTNetlinkError::RTNetlink)? {
        for nla in route_message.nlas.iter() {
            if let Nla::Oif(oif) = nla {
                if *oif == iface_idx {
                    let request = handle.route().del(route_message.clone());
                    request.execute().await.map_err(RTNetlinkError::RTNetlink)?;
                }
            }
        }
    }

    // Verify and return:
    match get_routes(handle).await?.len() {
        0 => Ok(()),
        _ => Err(RTNetlinkError::ValidationFailed),
    }
}

pub struct Ipv4Route {
    pub destination: Ipv4Addr,
    pub gateway: Ipv4Addr,
}

pub struct Ipv6Route {
    pub destination: Ipv6Addr,
    pub gateway: Ipv6Addr,
}

pub enum VersionOptions {
    V4(Ipv4Route),
    V6(Ipv6Route),
}

pub struct Route {
    pub iface_idx: u32,
    pub prefix_len: u8,
    pub version_opts: VersionOptions,
}

pub struct Scope {}
impl Scope {
    pub const UNIVERSE: u8 = 0;
    pub const SITE: u8 = 200;
    pub const LINK: u8 = 253;
    pub const HOST: u8 = 254;
    pub const NOWHERE: u8 = 255;
}

pub struct Table {}
impl Table {
    pub const UNSPEC: u32 = 0;
    pub const DEFAULT: u32 = 253;
    pub const LOCAL: u32 = 255;
    pub const MAIN: u32 = 254;
}

pub async fn add_route(handle: &Handle, route: Route) -> Result<(), RTNetlinkError> {
    // Although the function calls are the same, the type changes after .v4 or .v6.

    match &route.version_opts {
        VersionOptions::V4(v4_opts) => {
            debug!(
                "Adding {}/{} via {} on interface {}",
                v4_opts.destination, route.prefix_len, v4_opts.gateway, route.iface_idx
            );
            handle
                .route()
                .add()
                .v4()
                // Gateway and interface:
                .output_interface(route.iface_idx)
                .gateway(v4_opts.gateway)
                // Kernel address:
                .destination_prefix(v4_opts.destination, route.prefix_len)
                .table_id(243)
                .scope(Scope::UNIVERSE)
                .protocol(4)
                .execute()
                .await
                .map_err(|e| {
                    error!("add_route: RTNETLINK answers with error");
                    RTNetlinkError::RTNetlink(e)
                })?
        }
        VersionOptions::V6(v6_opts) => {
            debug!(
                "Adding {}/{} via {} on interface {}",
                v6_opts.destination, route.prefix_len, v6_opts.gateway, route.iface_idx
            );
            handle
                .route()
                .add()
                .v6()
                // Gateway and interface:
                .output_interface(route.iface_idx)
                .gateway(v6_opts.gateway)
                // Kernel address:
                .destination_prefix(v6_opts.destination, route.prefix_len)
                .execute()
                .await
                .map_err(|e| {
                    error!("add_route: RTNETLINK answers with error");
                    RTNetlinkError::RTNetlink(e)
                })?
        }
    };
    trace!("add route executed successfully");

    // Verify
    let ip_version = match route.version_opts {
        VersionOptions::V4(_) => IpVersion::V4,
        VersionOptions::V6(_) => IpVersion::V6,
    };

    let mut res_routes = handle.route().get(ip_version).execute();

    while let Some(route_message) = res_routes
        .try_next()
        .await
        .map_err(RTNetlinkError::RTNetlink)?
    {
        for nla in route_message.nlas.iter() {
            if let Nla::Oif(oif) = nla {
                if *oif == route.iface_idx {
                    debug!("Route added successfully!");
                    return Ok(());
                }
            }
        }
    }

    warn!("Route was not found after adding it.");
    Err(RTNetlinkError::ValidationFailed)
}

#[cfg(test)]
mod test_routes {
    use default_net::get_interfaces;

    #[tokio::test]
    async fn add_route() {
        pretty_env_logger::init();
        use super::*;
        use std::net::Ipv4Addr;

        let iface_idx = get_interfaces()
            .iter()
            .find(|iface| iface.name.starts_with('e'))
            .unwrap()
            .index;
        println!("Using interface index: {}", iface_idx);

        let (connection, handle, _) = rtnetlink::new_connection().unwrap();
        tokio::spawn(connection);

        let destination = Ipv4Addr::new(192, 168, 0, 0);
        let prefix_len = 24;
        let gateway = Ipv4Addr::new(192, 168, 0, 1);

        add_route(
            &handle,
            Route {
                iface_idx,
                prefix_len,
                version_opts: VersionOptions::V4(Ipv4Route {
                    destination,
                    gateway,
                }),
            },
        )
        .await
        .unwrap();
    }
}
