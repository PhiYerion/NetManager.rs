/* use crate::mac::set_mac;
use default_net::interface::get_interfaces;
use pnet::util::MacAddr;
use std::io::Error;
use std::io::ErrorKind::{AlreadyExists, NotFound};
use std::net::{IpAddr, Ipv4Addr};

struct VirtualIface {
    id: u32,
    name: String,
    mac: MacAddr,
    subnet_mask: u8,
    ipv4: Ipv4Addr,
    gateway: Ipv4Addr,
}

struct SubnetManager {
    virtual_ifaces: Vec<VirtualIface>,
    config_handler: NetworkConfigHandler,
}

enum IfaceIdentifier {
    ID(u32),
    Name(String),
}

// -- misc --
impl SubnetManager {
    pub fn new() -> Result<SubnetManager, Error> {
        Ok(SubnetManager {
            virtual_ifaces: Vec::new(),
            config_handler: NetworkConfigHandler::new()?,
        })
    }

    pub async fn flush_to_system(&self, iface_identifier: &IfaceIdentifier) -> Result<(), Error> {
        let iface = self.get_iface(iface_identifier)?;

        set_mac(&iface.name, iface.mac);
        self.config_handler
            .add_address(iface.id, IpAddr::from(iface.ipv4), iface.subnet_mask)
            .await?;
        self.config_handler
            .set_default_route(iface.id, iface.gateway)
            .await?;

        Ok(())
    }
}

// -- v-iface adder/getter --
impl SubnetManager {
    pub fn add_virtual_iface(
        &mut self,
        iface_identifier: IfaceIdentifier,
        mac: MacAddr,
        subnet_mask: u8,
        ipv4: Ipv4Addr,
        gateway: Ipv4Addr,
    ) -> Result<(), Error> {
        let (id, name) = get_iface_name_id_pair(&iface_identifier)?;

        if self.virtual_iface_exists(&iface_identifier) {
            return Err(Error::from(AlreadyExists));
        }

        self.virtual_ifaces.push(VirtualIface {
            id,
            name,
            mac,
            subnet_mask,
            ipv4,
            gateway,
        });

        Ok(())
    }

    pub fn get_iface_as_mut(
        &mut self,
        iface_identifier: &IfaceIdentifier,
    ) -> Result<&mut VirtualIface, Error> {
        Ok(match iface_identifier {
            IfaceIdentifier::ID(id) => self
                .virtual_ifaces
                .iter_mut()
                .find(|iface| iface.id == *id)
                .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) => self
                .virtual_ifaces
                .iter_mut()
                .find(|iface| iface.name == *name)
                .ok_or(NotFound)?,
        })
    }

    pub fn get_iface(&self, iface_identifier: &IfaceIdentifier) -> Result<&VirtualIface, Error> {
        Ok(match iface_identifier {
            IfaceIdentifier::ID(id) => self
                .virtual_ifaces
                .iter()
                .find(|&iface| iface.id == *id)
                .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) => self
                .virtual_ifaces
                .iter()
                .find(|&iface| iface.name == *name)
                .ok_or(NotFound)?,
        })
    }
}

// -- v-iface setters --
impl SubnetManager {
    pub fn set_iface_id(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        id: u32,
    ) -> Result<(), Error> {
        if self.virtual_iface_exists(iface_identifier) {
            return Err(Error::from(AlreadyExists));
        }

        let iface = self.get_iface_as_mut(iface_identifier)?;

        let (_, name) = get_iface_name_id_pair(&IfaceIdentifier::ID(id))?;
        iface.id = id;
        iface.name = name;

        Ok(())
    }

    pub fn set_iface_name(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        name: String,
    ) -> Result<(), Error> {
        if self.virtual_iface_exists(iface_identifier) {
            return Err(Error::from(AlreadyExists));
        }

        let iface = self.get_iface_as_mut(iface_identifier)?;

        (iface.id, _) = get_iface_name_id_pair(&IfaceIdentifier::Name(name))?;

        Ok(())
    }

    pub fn set_iface_mac(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        mac: MacAddr,
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.mac = mac;
        Ok(())
    }

    pub fn set_iface_subnet_mask(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        subnet_mask: u8,
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.subnet_mask = subnet_mask;
        Ok(())
    }

    pub fn set_iface_ipv4(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        ipv4: Ipv4Addr,
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.ipv4 = ipv4;
        Ok(())
    }

    pub fn set_iface_gateway(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        gateway: Ipv4Addr,
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.gateway = gateway;
        Ok(())
    }
}

// -- v-iface deleter --
impl SubnetManager {
    pub fn delete_iface(&mut self, iface_identifier: &IfaceIdentifier) -> Result<(), Error> {
        let index = match iface_identifier {
            IfaceIdentifier::ID(id) => self
                .virtual_ifaces
                .iter()
                .position(|iface| iface.id == *id)
                .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) => self
                .virtual_ifaces
                .iter()
                .position(|iface| iface.name == *name)
                .ok_or(NotFound)?,
        };

        self.virtual_ifaces.remove(index);
        Ok(())
    }
}

// -- Util --
impl SubnetManager {
    fn virtual_iface_exists(&self, iface_identifier: &IfaceIdentifier) -> bool {
        match iface_identifier {
            IfaceIdentifier::ID(id) => self.virtual_ifaces.iter().any(|iface| iface.id == *id),

            IfaceIdentifier::Name(name) => {
                self.virtual_ifaces.iter().any(|iface| iface.name == *name)
            }
        }
    }
}

// Other util:

pub fn iface_id_name_pair_matches(id: u32, name: String) -> bool {
    get_interfaces()
        .iter()
        .any(|iface| iface.index == id && iface.name == name)
}

pub fn get_iface_name_id_pair(iface_identifier: &IfaceIdentifier) -> Result<(u32, String), Error> {
    Ok(match iface_identifier {
        IfaceIdentifier::Name(name) => (
            // ID:
            get_interfaces()
                .iter()
                .find(|&iface| iface.name == *name)
                .ok_or(NotFound)?
                .index,
            // Name:
            name.clone(),
        ),
        IfaceIdentifier::ID(id) => (
            // ID:
            *id,
            // Name:
            get_interfaces()
                .iter()
                .find(|&iface| iface.index == *id)
                .ok_or(NotFound)?
                .name
                .clone(),
        ),
    })
} */
