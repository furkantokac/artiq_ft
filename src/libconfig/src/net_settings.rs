use core::fmt;

use libboard_zynq::smoltcp::wire::{EthernetAddress, IpAddress};

use super::Config;

pub struct NetAddresses {
    pub hardware_addr: EthernetAddress,
    pub ipv4_addr: IpAddress,
    #[cfg(feature = "ipv6")]
    pub ipv6_ll_addr: IpAddress,
    #[cfg(feature = "ipv6")]
    pub ipv6_addr: Option<IpAddress>
}

impl fmt::Display for NetAddresses {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MAC={} IPv4={} ",
            self.hardware_addr, self.ipv4_addr)?;

        #[cfg(feature = "ipv6")]
        {
            write!(f, "IPv6-LL={}", self.ipv6_ll_addr)?;
            match self.ipv6_addr {
                Some(addr) => write!(f, " {}", addr)?,
                None => write!(f, " IPv6: no configured address")?
            }
        }
        Ok(())
    }
}

pub fn get_adresses(cfg: &Config) -> NetAddresses {
    let mut hardware_addr = EthernetAddress([0x02, 0x00, 0x00, 0x00, 0x00, 0x52]);
    let mut ipv4_addr = IpAddress::v4(192, 168, 1, 52);

    if let Ok(Ok(addr)) = cfg.read_str("mac").map(|s| s.parse()) {
        hardware_addr = addr;
    }
    if let Ok(Ok(addr)) = cfg.read_str("ip").map(|s| s.parse()) {
        ipv4_addr = addr;
    }
    #[cfg(feature = "ipv6")]
    let ipv6_addr = cfg.read_str("ipv6").ok().and_then(|s| s.parse().ok());

    #[cfg(feature = "ipv6")]
    let ipv6_ll_addr = IpAddress::v6(
        0xfe80, 0x0000, 0x0000, 0x0000,
        (((hardware_addr.0[0] ^ 0x02) as u16) << 8) | (hardware_addr.0[1] as u16),
        ((hardware_addr.0[2] as u16) << 8) | 0x00ff,
        0xfe00 | (hardware_addr.0[3] as u16),
        ((hardware_addr.0[4] as u16) << 8) | (hardware_addr.0[5] as u16));

    NetAddresses {
        hardware_addr,
        ipv4_addr,
        #[cfg(feature = "ipv6")]
        ipv6_ll_addr,
        #[cfg(feature = "ipv6")]
        ipv6_addr
    }
}
