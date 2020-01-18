use log::info;
use params::{
    GetSetFlags, GetSetFrame, GetSetOp, GetSetPayloadType, Parameter, ParameterFlags, ParameterId,
    ParameterValue, Request, Response, NODE_ID_ANONYMOUS,
};
use std::io;
use std::io::prelude::*;
use std::net::SocketAddr;
use std::net::TcpStream;

pub fn set(address: SocketAddr, id: ParameterId, value: ParameterValue) -> io::Result<()> {
    info!("Set parameter ID {} Value {} at {}", id, value, address);
    let mut buf: Vec<u8> = vec![0; 1500];

    let mut frame = GetSetFrame::new_unchecked(&mut buf[..]);
    let mut req = Request::new(
        NODE_ID_ANONYMOUS,
        GetSetFlags::default(),
        GetSetOp::Set,
        GetSetPayloadType::ParameterListPacket,
    );
    let p = Parameter::new_with_value(id, ParameterFlags::default(), value);
    req.push_parameter(p).unwrap();
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
