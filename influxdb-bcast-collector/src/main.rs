use influx_db_client::{Client, Point, Points, Precision, Value};

// key == param.name
// field == param.value
// field.name = param.desc? or add a units/field-name toml?
//
// tags for node_id, zone, others?
// node.desc
//
// TODO
//
// measurement: param.id | "param.name"
// fields:
//   * "value" : param.value
// tags:
//   group by things?
//   * node_id | "node.name"
fn main() {
    let client = Client::new("http://localhost:8086", "parameters");
    //.set_authentication("root", "root");

    client.create_database("parameters").unwrap();

    for t_sec in 1..=10 {
        let mut point = Point::new("uptime");
        point.timestamp = Some(t_sec as i64);

        point.add_field("value", Value::Integer(t_sec as _));
        point.add_tag("node_id", Value::String("template_node1".into()));

        // if Precision is None, the default is second
        client
            .write_point(point, Some(Precision::Seconds), None)
            .unwrap();
    }
}
