use crate::error::Error;
use crate::time::Instant;
use log::debug;
use smoltcp::iface::EthernetInterface;
use smoltcp::socket::{SocketHandle, SocketSet, UdpSocket};
use smoltcp::wire::IpEndpoint;

pub const NEIGHBOR_CACHE_SIZE: usize = 32;
pub const SOCKET_BUFFER_SIZE: usize = 2048;

// 49152..=65535
const EPHEMERAL_PORT: u16 = 49152;

pub struct Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
    sockets: SocketSet<'d, 'e, 'f>,
    tcp_handle: SocketHandle,
    udp_handle: SocketHandle,
    udp_endpoint: IpEndpoint,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    pub fn new(
        iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
        sockets: SocketSet<'d, 'e, 'f>,
        tcp_handle: SocketHandle,
        udp_handle: SocketHandle,
        udp_endpoint: IpEndpoint,
    ) -> Result<Self, Error> {
        let mut eth = Eth {
            iface,
            sockets,
            tcp_handle,
            udp_handle,
            udp_endpoint,
        };

        debug!("UDP endpoint {}", eth.udp_endpoint);
        eth.sockets
            .get::<UdpSocket>(eth.udp_handle)
            .bind(EPHEMERAL_PORT)?;

        Ok(eth)
    }

    pub fn send_udp_bcast(&mut self, data: &[u8]) -> Result<(), Error> {
        self.sockets
            .get::<UdpSocket>(self.udp_handle)
            .send_slice(data, self.udp_endpoint)?;
        Ok(())
    }

    pub fn poll(&mut self, time: Instant) {
        let t = smoltcp::time::Instant::from_millis(time.as_millis() as i64);
        match self.iface.poll(&mut self.sockets, t) {
            Ok(true) => (),
            _ => (),
        }
    }
}
