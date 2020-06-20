use std::path::PathBuf;

use anyhow::Result;

use uclicious::raw::object::ObjectRef;

use uclicious_libucl_sys::ucl_type;

use umask::Mode;

use serde_json::map::Map;
use serde_json::Number;
use serde_json::Value;

use users::Group;
use users::User;

/// Convert an ObjectRef to a Value.
pub fn object_ref_to_value(src: ObjectRef) -> Result<Value> {
    match src.kind() {
        ucl_type::UCL_ARRAY => {
            let mut array: Vec<Value> = Vec::new();

            for item in src.iter() {
                let child: Value = object_ref_to_value(item)?;
                array.push(child);
            }

            Ok(Value::Array(array))
        }
        ucl_type::UCL_BOOLEAN => Ok(Value::Bool(src.as_bool().unwrap())),
        ucl_type::UCL_FLOAT => match Number::from_f64(src.as_f64().unwrap()) {
            Some(value) => Ok(Value::Number(value)),
            None => Err(anyhow!("cannot parse number {}", src.as_f64().unwrap())),
        },
        ucl_type::UCL_INT => Ok(Value::Number(Number::from(src.as_i64().unwrap()))),
        ucl_type::UCL_NULL => Ok(Value::Null),
        ucl_type::UCL_OBJECT => {
            let mut map: Map<String, Value> = Map::new();
            for item in src.iter() {
                let item_key = item.key().unwrap();
                let child: Value = object_ref_to_value(item)?;
                map.insert(item_key, child);
            }
            Ok(Value::Object(map))
        }
        ucl_type::UCL_STRING => Ok(Value::String(src.as_string().unwrap())),
        ucl_type::UCL_TIME => match Number::from_f64(src.as_time().unwrap()) {
            Some(value) => Ok(Value::Number(value)),
            None => Err(anyhow!("cannot parse number {}", src.as_f64().unwrap())),
        },
        ucl_type::UCL_USERDATA => Err(anyhow!("cannot convert userdata to json value")),
    }
}

#[cfg(target_os = "linux")]
pub fn get_user(path: &PathBuf) -> Result<User> {
    use std::os::linux::fs::MetadataExt;
    let uid = std::fs::File::open(path)?.metadata()?.st_uid();
    match users::get_user_by_uid(uid) {
        Some(value) => Ok(value),
        None => Err(anyhow!("no user found with the uid {}", uid)),
    }
}

#[cfg(target_os = "linux")]
pub fn get_group(path: &PathBuf) -> Result<Group> {
    use std::os::linux::fs::MetadataExt;
    let gid = std::fs::File::open(path)?.metadata()?.st_gid();
    match users::get_group_by_gid(gid) {
        Some(value) => Ok(value),
        None => Err(anyhow!("no group found with the gid {}", gid)),
    }
}

#[cfg(target_os = "linux")]
pub fn get_permissions(path: &PathBuf) -> Result<Mode> {
    use std::os::linux::fs::MetadataExt;
    Ok(Mode::from(
        std::fs::File::open(&path)?.metadata()?.st_mode(),
    ))
}
