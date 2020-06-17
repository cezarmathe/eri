use crate::config::ExportConfig;
use crate::data;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::convert::TryFrom;
use std::path::PathBuf;

use anyhow::Error;
use anyhow::Result;

use serde_json::Value;

use uclicious::raw::object::ObjectRef;

#[derive(Clone, Debug)]
pub struct NamespaceData(BTreeMap<String, Value>);

impl TryFrom<ObjectRef> for NamespaceData {
    type Error = Error;

    fn try_from(src: ObjectRef) -> Result<NamespaceData> {
        if src.is_null() {
            return Ok(NamespaceData(BTreeMap::default()));
        }
        if !src.is_object() {
            return Err(anyhow!("namespace must be an object"));
        }

        let mut map: BTreeMap<String, Value> = BTreeMap::new();

        for item in src.iter() {
            map.insert(item.key().unwrap(), data::object_ref_to_value(item)?);
        }

        Ok(NamespaceData(map))
    }
}

/// General representation of a template and data group
#[derive(Debug)]
pub struct Namespace<'a> {
    pub name: String,
    pub base_path: PathBuf,
    pub export_config: Cow<'a, ExportConfig>,
    pub data: NamespaceData
}

impl<'a> Namespace<'a> {
    /// Create a new namespace.
    pub fn new(name: &String, export_config: &'a ExportConfig, data: &NamespaceData) -> Result<Self> {
        let current_dir_path: PathBuf = match std::env::current_dir() {
            Ok(value) => value,
            Err(e) => panic!("cannot get the current directory: {:?}", e),
        };

        let base_path: PathBuf = current_dir_path.join(name);
        if !base_path.is_dir() {
            return Err(anyhow!("namespace {} does not have a directory", name));
        }

        Ok(Namespace {
            name: name.clone(),
            base_path,
            export_config: Cow::Borrowed(export_config),
            data: data.clone(),
        })
    }
}
