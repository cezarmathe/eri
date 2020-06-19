use std::path::PathBuf;

use anyhow::Result;

use uclicious::raw::object::ObjectRef;

use uclicious_libucl_sys::ucl_type;

use umask::Mode;

use serde_json::map::Map;
use serde_json::Number;
use serde_json::Value;

/// Convert an ObjectRef to a Value.
pub fn object_ref_to_value(src: ObjectRef) -> Result<Value> {
    log::trace!("converting object ref {:?} to value", src);
    match src.kind() {
        ucl_type::UCL_ARRAY => {
            log::trace!("ucl object is array");
            let mut array: Vec<Value> = Vec::new();

            for item in src.iter() {
                let child: Value = object_ref_to_value(item)?;
                array.push(child);
            }

            Ok(Value::Array(array))
        }
        ucl_type::UCL_BOOLEAN => {
            log::trace!("ucl object is bool");
            Ok(Value::Bool(src.as_bool().unwrap()))
        },
        ucl_type::UCL_FLOAT => {
            log::trace!("ucl object is float");
            match Number::from_f64(src.as_f64().unwrap()) {
                Some(value) => Ok(Value::Number(value)),
                None => Err(anyhow!("cannot parse number {}", src.as_f64().unwrap())),
            }
        },
        ucl_type::UCL_INT => {
            log::trace!("ucl object is integer");
            Ok(Value::Number(Number::from(src.as_i64().unwrap())))
        },
        ucl_type::UCL_NULL => {
            log::trace!("ucl object is null");
            Ok(Value::Null)
        },
        ucl_type::UCL_OBJECT => {
            log::trace!("ucl object is object");
            let mut map: Map<String, Value> = Map::new();
            for item in src.iter() {
                let item_key = item.key().unwrap();
                let child: Value = object_ref_to_value(item)?;
                map.insert(item_key, child);
            }
            Ok(Value::Object(map))
        }
        ucl_type::UCL_STRING => {
            log::trace!("ucl object is string");
            Ok(Value::String(src.as_string().unwrap()))
        },
        ucl_type::UCL_TIME => {
            log::trace!("ucl object is time");
            match Number::from_f64(src.as_time().unwrap()) {
                Some(value) => Ok(Value::Number(value)),
                None => Err(anyhow!("cannot parse number {}", src.as_f64().unwrap())),
            }
        },
        ucl_type::UCL_USERDATA => {
            log::warn!("ucl object is userdata");
            Err(anyhow!("cannot convert userdata to json value"))
        },
    }
}

#[cfg(target_os = "linux")]
pub fn get_user(path: &PathBuf) -> Result<String> {
    log::trace!("finding user for path {:?}", path);
    use std::os::linux::fs::MetadataExt;
    let uid = std::fs::File::open(path)?.metadata()?.st_uid();
    let user: users::User = match users::get_user_by_uid(uid) {
        Some(value) => {
            log::trace!("uid: {}", value.uid());
            value
        },
        None => {
            log::trace!("did not find a user with the uid {}", uid);
            return Err(anyhow!("no user found with the uid {}", uid))
        },
    };
    let user_string: String = match user.name().to_str() {
        Some(value) => {
            log::trace!("user: {}", value);
            value.to_owned()
        },
        None => {
            log::trace!("could not convert the user name to string");
            return Err(anyhow!("cannot retrieve user name from the uid {}", uid))
        },
    };
    Ok(user_string)
}

#[cfg(target_os = "linux")]
pub fn get_group(path: &PathBuf) -> Result<String> {
    log::trace!("finding user for path {:?}", path);
    use std::os::linux::fs::MetadataExt;
    let gid = std::fs::File::open(path)?.metadata()?.st_gid();
    let group: users::Group = match users::get_group_by_gid(gid) {
        Some(value) => {
            log::trace!("gid: {}", value.gid());
            value
        },
        None => {
            log::trace!("did not find a group with the gid {}", gid);
            return Err(anyhow!("no group found with the gid {}", gid))
        },
    };
    let group_string: String = match group.name().to_str() {
        Some(value) => {
            log::trace!("group: {}", value);
            value.to_owned()
        },
        None => {
            log::trace!("could not convert the group name to string");
            return Err(anyhow!("cannot retrieve group name from the gid {}", gid))
        },
    };
    Ok(group_string)
}

#[cfg(target_os = "linux")]
pub fn get_permissions(path: &PathBuf) -> Result<Mode> {
    log::trace!("finding permissions for path {:?}", path);
    use std::os::linux::fs::MetadataExt;
    Ok(Mode::from(
        std::fs::File::open(&path)?.metadata()?.st_mode(),
    ))
}
