use std::net::Ipv4Addr;
use futures::StreamExt;
use netlink_packet_core::{DoneMessage, ErrorMessage, NetlinkHeader, NetlinkMessage, NetlinkPayload, NLM_F_ACK, NLM_F_DUMP, NLM_F_MULTIPART, NLM_F_REQUEST};
use netlink_packet_route::{AF_INET, RouteHeader, RouteMessage, RT_SCOPE_SITE, RTM_DELROUTE, RTM_GETROUTE, RtnlMessage};
use netlink_packet_route::route::Nla::{Gateway, Oif, Table};
use netlink_packet_route::rule::Nla::Destination;
use netlink_sys::protocols::NETLINK_ROUTE;
use netlink_sys::{Socket, SocketAddr};
use rtnetlink::IpVersion;

pub fn send_with_response(pkt: &NetlinkMessage<RtnlMessage>) -> Result<Vec<NetlinkPayload<RtnlMessage>>, ErrorMessage> {

    let mut buf = vec![0; (pkt.header.length) as usize];
    pkt.serialize(&mut buf[..]);

    let socket = Socket::new(NETLINK_ROUTE).unwrap();
    let kernel_addr = SocketAddr::new(0, 0);

    let n_sent = socket.send_to(&buf[..], &kernel_addr, 0).unwrap();
    assert_eq!(n_sent, buf.len());
    let mut buf = vec![0; 8192];

    let mut payload_list: Vec<NetlinkPayload<RtnlMessage>> = Vec::new();
    loop {
        // receive a datagram
        let (n_received, sender_addr) = socket.recv_from(&mut &mut buf[..], 0).unwrap();
        assert_eq!(sender_addr, kernel_addr);
        let data = &buf[..n_received];
        println!("{:?}", data);
        let nlmsg = <NetlinkMessage<RtnlMessage>>::deserialize(&data).unwrap();
        dbg!(&nlmsg);
    //     match nlmsg.payload {
    //         NetlinkPayload::InnerMessage(..) => {
    //             payload_list.push(nlmsg.payload);
    //         }
    //         NetlinkPayload::Error(e) => {
    //             if e.code == None {
    //                 return Ok(vec!(NetlinkPayload::Done(DoneMessage::default())));
    //             } else {
    //                 return Err(e);
    //             }
    //         },
    //         NetlinkPayload::Done(p) => {
    //             return Ok(payload_list);
    //         }
    //         _ => {
    //             payload_list.push(nlmsg.payload);
    //         }
    //     }
    //     if nlmsg.header.flags != NLM_F_MULTIPART {
    //         return Ok(payload_list);
    //     }
    }
}

pub async fn get_routes_rt() {
    use rtnetlink::{new_connection, Error, Handle, IpVersion};
    use std::net::IpAddr;
    use tokio::runtime::Runtime;


    let (connection, handle, _) = new_connection().unwrap();
    tokio::spawn(connection);

    // Create a new netlink socket
    let oif_index = 2; // Output interface index
    let gateway = Ipv4Addr::new(192,168,4,1); // Gateway IP
    let destination = Ipv4Addr::new(0,0,0,0); // Destination IP
    let destination_prefix_length = 0; // Destination prefix length

    let mut request = handle.route().add().v4().replace();
    request = request.output_interface(oif_index);
    request = request.gateway(gateway);
    request = request.destination_prefix(destination, destination_prefix_length);
    request.execute().await.unwrap();
}

pub fn get_routes(interface_index: u32) -> Result<Vec<NetlinkPayload<RtnlMessage>>, ErrorMessage> {
    let mut header = NetlinkHeader::default();
    header.message_type = RTM_GETROUTE;
    header.flags = NLM_F_REQUEST | NLM_F_DUMP;
    header.sequence_number = 1;

    let mut payload_message = RouteMessage::default();
    payload_message.header.table = 254;

    let a = RtnlMessage::GetRoute(payload_message);

    let mut pkt = NetlinkMessage::new(
        header,
        a.into()
    );

    pkt.finalize();

    match send_with_response(&pkt) {
        Ok(r) => {
            Ok(r)
        },
        Err(e) => Err(e),
    }
}

pub fn flush_route(address_family: u8, interface_index: u32) -> Result<(), ErrorMessage> {
    let mut header = NetlinkHeader::default();
    header.message_type = RTM_DELROUTE;
    header.flags = NLM_F_REQUEST | NLM_F_ACK;

    let mut payload_message = RouteMessage::default();
    payload_message.header.address_family = address_family;
    payload_message.nlas.push(Oif(interface_index));

    let mut pkt = NetlinkMessage::new(
        header,
        RtnlMessage::DelRoute(payload_message).into()
    );

    pkt.finalize();

    match send_with_response(&pkt) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}