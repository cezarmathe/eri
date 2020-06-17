use std::collections::BTreeMap;
use std::convert::TryFrom;

use serde_json::Value;

use uclicious::*;
use uclicious_libucl_sys::ucl_type;

use umask::Mode;

pub struct Data(BTreeMap<String, Value>);

impl TryFrom<ObjectRef> for Data {
    type Error = ObjectError;

    fn try_from(src: ObjectRef) -> Result<Data, ObjectError> {
        if src.is_null() {
            return Ok(Value::Null);
        }


        for item in src.iter() {}
    }
}

pub fn map_data(src: ObjectRef) -> Result<BTreeMap<String, Value>, ObjectError> {
    if src.is_null() {
        return Ok(BTreeMap::default())
    }
    if !src.is_object() {
        return Err(ObjectError::Other("data is not an object".to_owned()))
    }

    let mut data: BTreeMap<String, Value> = BTreeMap::new();

    for item in src.iter() {
        match item.kind() {
            ucl_type::UCL_OBJECT => Value::Object
        }
    }

    Ok(data)
}

#[derive(Debug, Uclicious)]
pub struct Eri {
    export: Option<ExportConfig>,
    #[ucl(map = "map_data")]
    data: BTreeMap<String, Value>
}

#[derive(Debug, Uclicious)]
pub struct ExportConfig {
    dir: String,
    user: String,
    group: String,
    #[ucl(from="u32")]
    permissions: Mode,
}
