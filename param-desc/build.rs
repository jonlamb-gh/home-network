use serde_derive::Deserialize;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

// TODO
// - sanity check unique ID's, names, etc
// - sanitize name, snake case
// - ro comes with initial value
fn main() {
    let out_dir = Path::new(&std::env::var_os("OUT_DIR").unwrap()).to_owned();
    let src_dir = Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).to_owned();

    let toml = src_dir.join("paramdb.toml");
    let node_id_gen = out_dir.join("node_id_gen.rs");

    let toml_str = std::fs::read_to_string(toml).unwrap();
    let desc: Desc = toml::from_str(&toml_str).unwrap();

    let mut node_id_gen_file = File::create(node_id_gen).unwrap();

    desc.node.as_ref().map(|nodes| {
        nodes.iter().for_each(|n| {
            node_id_gen_file
                .write_all(n.gen_node_id().as_bytes())
                .unwrap()
        })
    });
}

//mod resources {
//    include! {concat!(env!("OUT_DIR"), "/resources.rs")}
//}

#[derive(Debug, Deserialize)]
struct Desc {
    node: Option<Vec<NodeDesc>>,
    parameter: Option<Vec<ParamDesc>>,
}

#[derive(Debug, Deserialize)]
struct NodeDesc {
    id: u32,
    name: String,
    desc: String,
}

#[derive(Debug, Deserialize)]
struct ParamDesc {
    id: u32,
    name: String,
    desc: String,
    value_type: String,
    value: Option<String>,
    ro: Option<bool>,
    bcast: Option<bool>,
}

impl NodeDesc {
    fn gen_node_id(&self) -> String {
        format!(
            "pub const {}: GetSetNodeId = {};\n",
            self.name.to_ascii_uppercase(),
            self.id
        )
    }
}
