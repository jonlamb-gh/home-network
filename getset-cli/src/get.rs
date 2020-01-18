use log::info;
use params::{
    GetSetFlags, GetSetFrame, GetSetOp, GetSetPayloadType, ParameterId, Request, Response,
    NODE_ID_ANONYMOUS,
};
use std::io;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;

pub fn get(address: SocketAddr, id: ParameterId) -> io::Result<()> {
    info!("Get parameter ID {} at {}", id, address);
    let mut buf: Vec<u8> = vec![0; 1500];

    let mut frame = GetSetFrame::new_unchecked(&mut buf[..]);
    let mut req = Request::new(
        NODE_ID_ANONYMOUS,
        GetSetFlags::default(),
        GetSetOp::Get,
        GetSetPayloadType::ParameterIdListPacket,
    );
    req.push_id(id).unwrap();
    req.emit(&mut frame).unwrap();
    let wire_size = req.wire_size();

    let mut stream = TcpStream::connect(address)?;

    info!("Sending {} bytes : {}", wire_size, req);
    stream.write(&buf[..wire_size])?;

    let bytes_read = stream.read(&mut buf[..])?;

    info!("Recv'd {} bytes", bytes_read);

    if let Ok(frame) = GetSetFrame::new_checked(&buf[..bytes_read]) {
        info!("{}", frame);
        if let Ok(resp) = Response::parse(&frame) {
            println!("{}", resp);
        }
    }

    Ok(())
}
