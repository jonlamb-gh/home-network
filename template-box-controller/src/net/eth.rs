use crate::error::Error;
use crate::time::Instant;
use log::debug;
use smoltcp::iface::EthernetInterface;
use smoltcp::socket::{SocketHandle, SocketSet, TcpSocket, TcpState, UdpSocket};
use smoltcp::wire::IpEndpoint;

pub const NEIGHBOR_CACHE_SIZE: usize = 32;
pub const SOCKET_BUFFER_SIZE: usize = 2048;
pub const MTU: usize = 1500;

const TCP_TIMEOUT_DURATION: Option<smoltcp::time::Duration> =
    Some(smoltcp::time::Duration { millis: 5 * 1000 });
const TCP_KEEP_ALIVE_INTERVAL: Option<smoltcp::time::Duration> =
    Some(smoltcp::time::Duration { millis: 2 * 1000 });

// 49152..=65535
const EPHEMERAL_PORT: u16 = 49152;

pub struct Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
    sockets: SocketSet<'d, 'e, 'f>,
    udp_handle: SocketHandle,
    udp_endpoint: IpEndpoint,
    tcp_handle: SocketHandle,
    tcp_endpoint: IpEndpoint,
    tcp_was_connected: bool,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> Eth<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx, 'r> {
    pub fn new(
        iface: EthernetInterface<'a, 'b, 'c, &'r mut stm32_eth::Eth<'rx, 'tx>>,
        sockets: SocketSet<'d, 'e, 'f>,
        tcp_handle: SocketHandle,
        tcp_endpoint: IpEndpoint,
        udp_handle: SocketHandle,
        udp_endpoint: IpEndpoint,
    ) -> Result<Self, Error> {
        let mut eth = Eth {
            iface,
            sockets,
            udp_handle,
            udp_endpoint,
            tcp_handle,
            tcp_endpoint,
            tcp_was_connected: false,
        };

        debug!("UDP endpoint {}", eth.udp_endpoint);
        eth.sockets
            .get::<UdpSocket>(eth.udp_handle)
            .bind(EPHEMERAL_PORT)?;

        eth.listen();

        Ok(eth)
    }

    pub fn send_udp_bcast(&mut self, data: &[u8]) -> Result<(), Error> {
        self.sockets
            .get::<UdpSocket>(self.udp_handle)
            .send_slice(data, self.udp_endpoint)?;
        Ok(())
    }

    // TODO - send/recv TCP data fn's
    pub fn recv_tcp(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        let mut socket = self.sockets.get::<TcpSocket>(self.tcp_handle);
        if socket.may_recv() {
            let bytes_recvd = socket.recv_slice(data)?;
            Ok(bytes_recvd)
        } else {
            Ok(0)
        }
    }

    pub fn poll(&mut self, time: Instant) {
        let mut relisten = false;
        let t = smoltcp::time::Instant::from_millis(time.as_millis() as i64);
        match self.iface.poll(&mut self.sockets, t) {
            Ok(true) => {
                // Something happened, manage the TCP server socket
                let mut socket = self.sockets.get::<TcpSocket>(self.tcp_handle);

                let remote_disconnected = if socket.is_active()
                    && self.tcp_was_connected
                    && (socket.state() == TcpState::CloseWait)
                {
                    true
                } else {
                    false
                };

                if socket.is_active() && !self.tcp_was_connected {
                    debug!("TCP connected");
                    self.tcp_was_connected = true
                } else if (!socket.is_active() && self.tcp_was_connected) || remote_disconnected {
                    debug!("TCP disconnected");
                    self.tcp_was_connected = false;
                    if remote_disconnected {
                        socket.close();
                    }
                    socket.abort();
                    relisten = true;
                }
            }
            _ => (),
        }

        if relisten {
            self.listen();
        }
    }

    fn listen(&mut self) {
        let mut socket = self.sockets.get::<TcpSocket>(self.tcp_handle);
        socket.abort();
        debug!("TCP endpoint listening on port {}", self.tcp_endpoint.port);
        socket
            .listen(self.tcp_endpoint.port)
            .expect("TCP socket already open");
        socket.set_timeout(TCP_TIMEOUT_DURATION);
        socket.set_keep_alive(TCP_KEEP_ALIVE_INTERVAL);
    }
}
