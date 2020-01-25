use params::ParameterValueTypeId;
use serde_derive::Deserialize;
use std::collections::HashSet;
use std::fs::File;
use std::hash::Hash;
use std::io::prelude::*;
use std::path::Path;
use std::str::FromStr;

// TODO
// - sanity check unique ID's, names, etc
// - sanitize name, snake case
// - ro comes with initial value
fn main() {
    let out_dir = Path::new(&std::env::var_os("OUT_DIR").unwrap()).to_owned();
    let src_dir = Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).to_owned();

    let toml = src_dir.join("paramdb.toml");
    let node_id_gen = out_dir.join("node_id_gen.rs");
    let node_name_gen = out_dir.join("node_name_gen.rs");
    let node_desc_gen = out_dir.join("node_desc_gen.rs");
    let param_id_gen = out_dir.join("param_id_gen.rs");
    let param_name_gen = out_dir.join("param_name_gen.rs");
    let param_desc_gen = out_dir.join("param_desc_gen.rs");
    let param_gen = out_dir.join("param_gen.rs");

    println!("rerun-if-changed={}", toml.display());

    let toml_str = std::fs::read_to_string(toml).unwrap();
    let desc: Desc = toml::from_str(&toml_str).unwrap();

    desc.node.as_ref().map(|nodes| {
        let ids: Vec<u32> = nodes.iter().map(|p| p.id).collect();
        assert!(has_unique_elements(ids));
    });

    desc.parameter.as_ref().map(|params| {
        let ids: Vec<u32> = params.iter().map(|p| p.id).collect();
        assert!(has_unique_elements(ids));
    });

    // Generate node ID/desc/name
    let mut node_id_gen_file = File::create(node_id_gen).unwrap();
    let mut node_name_gen_file = File::create(node_name_gen).unwrap();
    let mut node_desc_gen_file = File::create(node_desc_gen).unwrap();

    desc.node.as_ref().map(|nodes| {
        nodes.iter().for_each(|n| {
            node_id_gen_file.write_all(n.gen_id().as_bytes()).unwrap();
            node_desc_gen_file
                .write_all(n.gen_desc().as_bytes())
                .unwrap();
        })
    });

    node_desc_gen_file
        .write_all(
            r#"
pub fn node_desc(id: GetSetNodeId) -> Option<&'static str> {
match id {
"#
            .as_bytes(),
        )
        .unwrap();
    desc.node.as_ref().map(|nodes| {
        nodes.iter().for_each(|n| {
            node_desc_gen_file
                .write_all(format!("{} => Some(\"{}\"),\n", n.id, n.desc).as_bytes())
                .unwrap()
        })
    });
    node_desc_gen_file.write_all(b"_ => None,\n").unwrap();
    node_desc_gen_file.write_all(b"}}\n").unwrap();

    node_name_gen_file
        .write_all(
            r#"
pub fn node_name(id: GetSetNodeId) -> Option<&'static str> {
match id {
"#
            .as_bytes(),
        )
        .unwrap();
    desc.node.as_ref().map(|nodes| {
        nodes.iter().for_each(|n| {
            node_name_gen_file
                .write_all(format!("{} => Some(\"{}\"),\n", n.id, n.name).as_bytes())
                .unwrap()
        })
    });
    node_name_gen_file.write_all(b"_ => None,\n").unwrap();
    node_name_gen_file.write_all(b"}}\n").unwrap();

    // Generate parameter ID/name/desc
    let mut param_id_gen_file = File::create(param_id_gen).unwrap();
    let mut param_name_gen_file = File::create(param_name_gen).unwrap();
    let mut param_desc_gen_file = File::create(param_desc_gen).unwrap();

    desc.parameter.as_ref().map(|params| {
        params.iter().for_each(|p| {
            param_id_gen_file.write_all(p.gen_id().as_bytes()).unwrap();
            param_desc_gen_file
                .write_all(p.gen_desc().as_bytes())
                .unwrap();
        })
    });

    param_desc_gen_file
        .write_all(
            r#"
pub fn param_desc(id: ParameterId) -> Option<&'static str> {
match id.0 {
"#
            .as_bytes(),
        )
        .unwrap();
    desc.parameter.as_ref().map(|params| {
        params.iter().for_each(|p| {
            param_desc_gen_file
                .write_all(format!("{} => Some(\"{}\"),\n", p.id, p.desc).as_bytes())
                .unwrap();
        })
    });
    param_desc_gen_file.write_all(b"_ => None,\n").unwrap();
    param_desc_gen_file.write_all(b"}}\n").unwrap();

    param_name_gen_file
        .write_all(
            r#"
pub fn param_name(id: ParameterId) -> Option<&'static str> {
match id.0 {
"#
            .as_bytes(),
        )
        .unwrap();
    desc.parameter.as_ref().map(|params| {
        params.iter().for_each(|p| {
            param_name_gen_file
                .write_all(format!("{} => Some(\"{}\"),\n", p.id, p.name).as_bytes())
                .unwrap();
        })
    });
    param_name_gen_file.write_all(b"_ => None,\n").unwrap();
    param_name_gen_file.write_all(b"}}\n").unwrap();

    // Generate parameter consts
    let mut param_gen_file = File::create(param_gen).unwrap();

    desc.parameter.as_ref().map(|params| {
        params.iter().for_each(|p| {
            param_gen_file.write_all(p.gen_param().as_bytes()).unwrap();
        })
    });
}

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

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
struct ParamDesc {
    id: u32,
    name: String,
    desc: String,
    value_type: String,
    value: Option<String>,
    ro: Option<bool>,
    bcast: Option<bool>,
    constant: Option<bool>,
}

impl NodeDesc {
    fn gen_id(&self) -> String {
        format!(
            "pub const {}: GetSetNodeId = {};\n",
            self.name.to_ascii_uppercase(),
            self.id
        )
    }

    fn gen_desc(&self) -> String {
        format!(
            "pub const {}_DESC: &'static str = \"{}\";\n",
            self.name.to_ascii_uppercase(),
            self.desc,
        )
    }
}

impl ParamDesc {
    fn gen_id(&self) -> String {
        format!(
            "pub const {}: ParameterId = ParameterId::new({});\n",
            self.name.to_ascii_uppercase(),
            self.id
        )
    }

    fn gen_desc(&self) -> String {
        format!(
            "pub const {}_DESC: &'static str = \"{}\";\n",
            self.name.to_ascii_uppercase(),
            self.desc,
        )
    }

    fn gen_param(&self) -> String {
        let value_type = ParameterValueTypeId::from_str(&self.value_type).unwrap();

        let value = match value_type {
            ParameterValueTypeId::None => String::from("ParameterValue::None"),
            ParameterValueTypeId::Notification => String::from("ParameterValue::Notification"),
            ParameterValueTypeId::Bool => String::from(format!(
                "ParameterValue::Bool({})",
                bool::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::U8 => String::from(format!(
                "ParameterValue::U8({})",
                u8::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::I8 => String::from(format!(
                "ParameterValue::I8({})",
                i8::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::U32 => String::from(format!(
                "ParameterValue::U32({})",
                u32::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::I32 => String::from(format!(
                "ParameterValue::I32({})",
                i32::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::U64 => String::from(format!(
                "ParameterValue::U64({})",
                u64::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::I64 => String::from(format!(
                "ParameterValue::I64({})",
                i64::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
            ParameterValueTypeId::F32 => String::from(format!(
                "ParameterValue::F32({}_f32)",
                f32::from_str(self.value.as_ref().unwrap()).unwrap()
            )),
        };

        let ro = self.ro.unwrap_or(false);
        let bcast = self.bcast.unwrap_or(false);
        let constant = self.constant.unwrap_or(false);

        let flags = String::from(format!(
            "ParameterFlags::new_from_flags({} | {} | {})",
            if ro { "RO" } else { "0" },
            if bcast { "BCAST" } else { "0" },
            if constant { "CONST" } else { "0" },
        ));

        format!(
            r#"
pub const {}: Parameter = Parameter::new_with_value(
    ParameterId::new({}),
    {},
    {}
);
"#,
            self.name.to_ascii_uppercase(),
            self.id,
            flags,
            value,
        )
    }
}

fn has_unique_elements<T>(iter: T) -> bool
where
    T: IntoIterator,
    T::Item: Eq + Hash,
{
    let mut uniq = HashSet::new();
    iter.into_iter().all(move |x| uniq.insert(x))
}
