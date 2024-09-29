//
// TCP layer for underlying consensus library
//
// SEP 27th - flatbuffers or protobuf for rpc?
//

use std::net::Ipv4Addr;
use std::net::TcpListener;
use std::str::FromStr;

#[derive(Debug)]
pub struct ConnectionLayer {
    local_addr: Ipv4Addr,
    listener: *mut TcpListener,
}

#[allow(unused)]
impl ConnectionLayer {
    pub fn init_layer(addr: &str) -> ConnectionLayer {
        // addr being str and IPv4 is a BIG NONO
        let local_addr: Ipv4Addr;
        match Ipv4Addr::from_str(addr) {
            Ok(addr) => local_addr = addr,
            Err(e) => panic!("{}", e),
        }
        // i guess that is fine
        let mut listener = TcpListener::bind(addr).unwrap();
        return ConnectionLayer {
            local_addr,
            listener: &mut listener,
        };
    }
}
