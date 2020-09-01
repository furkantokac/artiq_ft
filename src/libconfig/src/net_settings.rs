use core::fmt;

use libboard_zynq::smoltcp::wire::{EthernetAddress, IpAddress};
use super::Config;

pub struct NetAddresses {
    pub hardware_addr: EthernetAddress,
    pub ipv4_addr: IpAddress,
    pub ipv6_ll_addr: IpAddress,
    pub ipv6_addr: Option<IpAddress>
}

impl fmt::Display for NetAddresses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MAC={} IPv4={} IPv6-LL={} IPv6=",
            self.hardware_addr, self.ipv4_addr, self.ipv6_ll_addr)?;
        match self.ipv6_addr {
            Some(addr) => write!(f, "{}", addr)?,
            None => write!(f, "no configured address")?
        }
        Ok(())
    }
}

pub fn get_adresses(cfg: &Config) -> NetAddresses {
    let mut hardware_addr = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x52]);
    let mut ipv4_addr = IpAddress::v4(192, 168, 1, 52);
    let mut ipv6_addr = None;

    if let Ok(Ok(addr)) = cfg.read_str("mac").map(|s| s.parse()) {
        hardware_addr = addr;
    }
    if let Ok(Ok(addr)) = cfg.read_str("ip").map(|s| s.parse()) {
        ipv4_addr = addr;
    }
    if let Ok(Ok(addr)) = cfg.read_str("ip6").map(|s| s.parse()) {
        ipv6_addr = Some(addr);
    }

    let ipv6_ll_addr = IpAddress::v6(
        0xfe80, 0x0000, 0x0000, 0x0000,
        (((hardware_addr.0[0] ^ 0x02) as u16) << 8) | (hardware_addr.0[1] as u16),
        ((hardware_addr.0[2] as u16) << 8) | 0x00ff,
        0xfe00 | (hardware_addr.0[3] as u16),
        ((hardware_addr.0[4] as u16) << 8) | (hardware_addr.0[5] as u16));

    NetAddresses {
        hardware_addr: hardware_addr,
        ipv4_addr: ipv4_addr,
        ipv6_ll_addr: ipv6_ll_addr,
        ipv6_addr: ipv6_addr
    }
}
