use log::info;
use params::{GetSetFrame, Response};
use std::io;
use std::net::SocketAddr;
use std::net::UdpSocket;

pub fn start_listening(address: SocketAddr) -> io::Result<()> {
    let socket = UdpSocket::bind(address)?;

    let mut buf: Vec<u8> = vec![0; 1500];

    info!("Listening for broadcast GetSetFrame's on {}", address);
    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        info!("Got {} bytes from {}", amt, src);

        if amt >= GetSetFrame::<&[u8]>::header_len() {
            if let Ok(frame) = GetSetFrame::new_checked(&buf[..amt]) {
                info!("{}", frame);
                if let Ok(resp) = Response::parse(&frame) {
                    info!("{}", resp);
                }
            }
        }
    }
}
