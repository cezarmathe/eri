use std::collections::BTreeMap;
use std::convert::TryFrom;

use serde_json::Value;
use uclicious::*;
use umask::Mode;

fn map_mode(src: ObjectRef) -> Result<Mode, ObjectError> {
    let val: u32 = src.try_into()?;
    let user_value: u32 = val / 100;
    let group_value: u32 = val % 100 / 10;
    let all_value: u32 = val % 10;
    Ok(Mode::from(user_value * 64 + group_value * 8 + all_value))
}

#[derive(Debug, Uclicious)]
pub struct ExportConfig {
    dir: String,
    user: String,
    group: String,
    #[ucl(map = "map_mode")]
    permissions: Mode,
}

pub type IncludeConfig = Vec<String>;

#[derive(Debug)]
pub struct Namespace(BTreeMap<String, Value>);

pub type Namespaces = BTreeMap<String, Namespace>;

impl TryFrom<ObjectRef> for Namespace {
    type Error = ObjectError;

    fn try_from(src: ObjectRef) -> Result<Namespace, ObjectError> {
        Ok(Namespace(BTreeMap::default()))
    }
}

fn map_object_ref_to_optional_namespace_btree(
    src: ObjectRef,
) -> Result<Option<Namespaces>, ObjectError> {
    Ok(None)
}

#[derive(Debug, Uclicious)]
pub struct EriConfig {
    #[ucl(default)]
    pub export: Option<ExportConfig>,
    #[ucl(default)]
    pub include: Option<IncludeConfig>,
    // #[ucl(map = "map_object_ref_to_optional_namespace_btree")]
    // pub namespaces: Option<Namespaces>,
}
