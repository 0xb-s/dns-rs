use std::net::{Ipv4Addr, Ipv6Addr};

pub fn ipv4_to_bytes(ip: &Ipv4Addr) -> [u8; 4] {
    ip.octets()
}

pub fn ipv6_to_bytes(ip: &Ipv6Addr) -> [u8; 16] {
    ip.octets()
}
