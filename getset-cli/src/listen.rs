use log::info;
use std::io;
use std::net::SocketAddr;
use std::net::UdpSocket;

pub fn start_listening(address: SocketAddr) -> io::Result<()> {
    let socket = UdpSocket::bind(address)?;

    let mut buf: Vec<u8> = vec![0; 1500];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        info!("Got {} bytes from {}", amt, src);
    }
}
