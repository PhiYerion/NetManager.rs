use std::io::Error;
use std::io::ErrorKind::{AlreadyExists, NotFound};
use std::net::Ipv4Addr;
use pnet::util::MacAddr;
use default_net::interface::get_interfaces;

struct Virtualiface {
    id: u32,
    name: String,
    mac: MacAddr,
    subnet_mask: u8,
    ipv4: Ipv4Addr,
    gateway: Ipv4Addr,
}

struct SubnetManager {
    virtual_ifaces: Vec<Virtualiface>
}

enum IfaceIdentifier {
    ID(u32),
    Name(String)
}

// -- iface adder/getter --
impl SubnetManager {
    pub fn add_virtual_iface(
        &mut self,
        iface_identifier: IfaceIdentifier,
        mac: MacAddr,
        subnet_mask: u8,
        ipv4: Ipv4Addr,
        gateway: Ipv4Addr
    ) -> Result<(), Error> {
        let (id, name) = get_iface_name_id_pair(&iface_identifier)?;

        if self.virtual_iface_exists(&iface_identifier) {
            return Err(Error::from(AlreadyExists));
        }

        self.virtual_ifaces.push(Virtualiface {
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
        iface_identifier: &IfaceIdentifier
    ) -> Result<&mut Virtualiface, Error> {
        Ok(match iface_identifier {
            IfaceIdentifier::ID(id) =>
                self.virtual_ifaces
                    .iter_mut()
                    .find(|&iface| iface.id == *id)
                    .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) =>
                self.virtual_ifaces
                    .iter_mut()
                    .find(|&iface| iface.name == *name)
                    .ok_or(NotFound)?
        })
    }

    pub fn get_iface(
        &mut self,
        iface_identifier: &IfaceIdentifier
    ) -> Result<&Virtualiface, Error> {
        Ok(match iface_identifier {
            IfaceIdentifier::ID(id) =>
                self.virtual_ifaces
                    .iter()
                    .find(|&iface| iface.id == *id)
                    .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) =>
                self.virtual_ifaces
                    .iter()
                    .find(|&iface| iface.name == *name)
                    .ok_or(NotFound)?
        })
    }
}

// -- iface setters --
impl SubnetManager {
    pub fn set_iface_id(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        id: u32
    ) -> Result<(), Error> {
        if self.virtual_iface_exists(&iface_identifier) {
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
        name: String
    ) -> Result<(), Error> {
        if self.virtual_iface_exists(&iface_identifier) {
            return Err(Error::from(AlreadyExists));
        }

        let iface = self.get_iface_as_mut(iface_identifier)?;

        (iface.id, _) = get_iface_name_id_pair(&IfaceIdentifier::Name(name))?;

        Ok(())
    }

    pub fn set_iface_mac(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        mac: MacAddr
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.mac = mac;
        Ok(())
    }

    pub fn set_iface_subnet_mask(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        subnet_mask: u8
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.subnet_mask = subnet_mask;
        Ok(())
    }

    pub fn set_iface_ipv4(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        ipv4: Ipv4Addr
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.ipv4 = ipv4;
        Ok(())
    }

    pub fn set_iface_gateway(
        &mut self,
        iface_identifier: &IfaceIdentifier,
        gateway: Ipv4Addr
    ) -> Result<(), Error> {
        let iface = self.get_iface_as_mut(iface_identifier)?;
        iface.gateway = gateway;
        Ok(())
    }
}

// -- iface deleter --
impl SubnetManager {
    pub fn delete_iface(
        &mut self,
        iface_identifier: &IfaceIdentifier
    ) -> Result<(), Error> {
        let index = match iface_identifier {
            IfaceIdentifier::ID(id) =>
                self.virtual_ifaces
                    .iter()
                    .position(|iface| iface.id == *id)
                    .ok_or(NotFound)?,

            IfaceIdentifier::Name(name) =>
                self.virtual_ifaces
                    .iter()
                    .position(|iface| iface.name == *name)
                    .ok_or(NotFound)?
        };

        self.virtual_ifaces.remove(index);
        Ok(())
    }
}

// -- Util --
impl SubnetManager {
    fn virtual_iface_exists(
        &self,
        iface_identifier: &IfaceIdentifier
    ) -> bool {
        match iface_identifier {
            IfaceIdentifier::ID(id) =>
                self.virtual_ifaces.iter()
                    .find(|&iface| iface.id == *id)
                    .is_some(),

            IfaceIdentifier::Name(name) =>
                self.virtual_ifaces
                    .iter()
                    .find(|&iface| iface.name == *name)
                    .is_some()
        }
    }
}

pub fn iface_id_name_pair_matches(id: u32, name: String) -> bool {
    get_interfaces()
        .iter()
        .find(|&iface| iface.index == id && iface.name == name)
        .is_some()
}

pub fn get_iface_name_id_pair(
    iface_identifier: &IfaceIdentifier
) -> Result<(u32, String), Error> {
    Ok(match iface_identifier {
        IfaceIdentifier::Name(name) => (
            // ID:
            get_interfaces()
                .iter()
                .find(|&iface| iface.name == *name)
                .ok_or(NotFound)?.index,
            // Name:
            name.clone()
        ),
        IfaceIdentifier::ID(id) => (
            // ID:
            *id,
            // Name:
            get_interfaces()
                .iter()
                .find(|&iface| iface.index == *id)
                .ok_or(NotFound)?.name.clone()
        )
    })
}