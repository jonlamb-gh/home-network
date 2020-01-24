use influx_db_client::{Client, Point, Precision, Value};
use log::info;
use param_desc::node_name::node_name;
use param_desc::param_name::param_name;
use params::{GetSetFrame, ParameterValue, Response};
use std::io;
use std::net::SocketAddr;
use std::net::UdpSocket;

// TODO - should precision be set to seconds or ms?
// should local_time_ms be another tag/field?
pub fn start_listening(address: SocketAddr, client: String, db: String) -> io::Result<()> {
    info!("Setup client at {}, database '{}'", client, db);

    let client = Client::new(client.to_string(), db.clone());
    //.set_authentication("root", "root");

    client.create_database(&db).unwrap();

    info!("Listening for broadcast GetSetFrame's on {}", address);
    let socket = UdpSocket::bind(address)?;
    let mut buf: Vec<u8> = vec![0; 1500];

    loop {
        let (amt, src) = socket.recv_from(&mut buf)?;
        info!("Got {} bytes from {}", amt, src);

        if amt >= GetSetFrame::<&[u8]>::header_len() {
            if let Ok(frame) = GetSetFrame::new_checked(&buf[..amt]) {
                info!("{}", frame);
                let node_id = frame.node_id();
                let node_name =
                    node_name(node_id).map_or(format!("Unkown({})", node_id), |s| String::from(s));
                if let Ok(resp) = Response::parse(&frame) {
                    println!("{}", resp);
                    for p in resp.parameters() {
                        let param_name = param_name(p.id())
                            .map_or(format!("Unkown({})", p.id()), |s| String::from(s));
                        let mut point = Point::new(&param_name);
                        point.timestamp = Some(p.local_time_ms() as i64);
                        let val = match p.value() {
                            ParameterValue::None => Value::String(String::from("None")),
                            ParameterValue::Notification => {
                                Value::String(String::from("Notification"))
                            }
                            ParameterValue::Bool(v) => Value::Boolean(v),
                            ParameterValue::U8(v) => Value::Integer(v as i64),
                            ParameterValue::U32(v) => Value::Integer(v as i64),
                            ParameterValue::I32(v) => Value::Integer(v as i64),
                            ParameterValue::F32(v) => Value::Float(v.into()),
                        };
                        point.add_field("value", val);
                        point.add_tag("node_id", Value::String(node_name.clone()));
                        info!("Logging {:?}", point);
                        client
                            .write_point(point, Some(Precision::Milliseconds), None)
                            .unwrap();
                    }
                }
            }
        }
    }
}
