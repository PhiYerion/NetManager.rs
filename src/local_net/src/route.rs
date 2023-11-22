use futures::{TryStreamExt, Future};
use netlink_packet_route::RouteMessage;
use netlink_packet_route::route::Nla;
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
    Fut: Future<Output = Result<(), RTNetlinkError>> {
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
    modify_routes(handle, ip_version.clone(), async move |route_message| -> Result<(), RTNetlinkError> {
        for nla in route_message.nlas.iter() {
            if let Nla::Oif(oif) = nla {
                if *oif == iface_idx {
                    let request = handle.route().del(route_message.clone());
                    request.execute().await.map_err(RTNetlinkError::RTNetlink)?;
                }
            }
        }
        Ok(())
    }).await?;

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

pub async fn add_route(
    handle: &Handle,
    route: Route,
) -> Result<(), RTNetlinkError> {
    // Although the function calls are the same, the type changes after .v4 or .v6.
    match &route.version_opts {
        VersionOptions::V4(v4_opts) => {
                handle
                .route()
                .add()
                .v4()
                // Gateway and interface:
                .output_interface(route.iface_idx)
                .gateway(v4_opts.gateway)
                // Kernel address:
                .destination_prefix(v4_opts.destination, route.prefix_len)
                .execute().await.map_err(RTNetlinkError::RTNetlink)?
        },
        VersionOptions::V6(v6_opts) => {
                handle
                .route()
                .add()
                .v6()
                // Gateway and interface:
                .output_interface(route.iface_idx)
                .gateway(v6_opts.gateway)
                // Kernel address:
                .destination_prefix(v6_opts.destination, route.prefix_len)
                .execute().await.map_err(RTNetlinkError::RTNetlink)?
        },
    };

    // Verify
    let ip_version = match route.version_opts {
        VersionOptions::V4(_) => IpVersion::V4,
        VersionOptions::V6(_) => IpVersion::V6,
    };

    let mut res_routes = handle
        .route()
        .get(ip_version)
        .execute();

    while let Some(route_message) = res_routes.try_next().await.map_err(RTNetlinkError::RTNetlink)? {
        for nla in route_message.nlas.iter() {
            if let Nla::Oif(oif) = nla {
                if *oif == route.iface_idx {
                    return Err(RTNetlinkError::ValidationFailed);
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test_routes {
    use default_net::get_interfaces;

    #[tokio::test]
    async fn tester() {
        use super::*;
        use std::net::Ipv4Addr;

        let iface_idx = get_interfaces().iter().find(|iface| 
            iface.name.starts_with('e')
        ).unwrap().index;

        let (connection, handle, _) = rtnetlink::new_connection().unwrap();
        tokio::spawn(connection);

        let destination = Ipv4Addr::new(10, 0, 0, 0);
        let prefix_len = 8;
        let gateway = Ipv4Addr::new(10, 0, 0, 1);

        let res = add_route(
            &handle,
            Route {
                iface_idx,
                prefix_len,
                version_opts: VersionOptions::V4(Ipv4Route {
                    destination,
                    gateway,
                }),
            }
        )
        .await;

        assert!(res.is_ok());
    }
}
