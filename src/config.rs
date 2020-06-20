use crate::data;
use crate::namespace::Namespace;

use std::borrow::Cow;
use std::path::PathBuf;

use anyhow::Result;

use serde_json::Map;
use serde_json::Value;

use uclicious::raw::object::ObjectError;
use uclicious::raw::object::ObjectRef;
use uclicious::raw::Priority;
use uclicious::DEFAULT_DUPLICATE_STRATEGY;

use uclicious_derive::*;

use uclicious_libucl_sys::ucl_type;

use umask::Mode;

use users::Group;
use users::User;

/// Map permissions mode.
fn map_mode(src: ObjectRef) -> Result<Option<Mode>, ObjectError> {
    if src.is_null() {
        return Ok(None);
    }
    if !src.is_integer() {
        return Err(ObjectError::WrongType {
            key: "permissions".to_owned(),
            actual_type: src.kind(),
            wanted_type: ucl_type::UCL_INT,
        });
    }

    use uclicious::TryInto;
    let val: u32 = src.try_into()?;

    let user_value: u32 = val / 100;
    let group_value: u32 = val % 100 / 10;
    let all_value: u32 = val % 10;

    Ok(Some(Mode::from(
        user_value * 64 + group_value * 8 + all_value,
    )))
}

/// Map the eri config namespaces from ucl.
fn map_namespace(src: ObjectRef) -> Result<Map<String, Value>, ObjectError> {
    let mut result: Map<String, Value> = Map::new();

    for item in src.iter() {
        let item_key = item.key().unwrap();
        match data::object_ref_to_value(item) {
            Ok(value) => result.insert(item_key, value),
            Err(e) => {
                return Err(ObjectError::Other(e.to_string()));
            }
        };
    }

    Ok(result)
}

/// Map an eri config user to an actual user.
fn map_user(src: ObjectRef) -> Result<Option<User>, ObjectError> {
    if src.is_null() {
        return Ok(None);
    }
    match src.kind() {
        ucl_type::UCL_STRING => {
            let username: String = src.as_string().unwrap();
            if let Some(value) = users::get_user_by_name(&username) {
                Ok(Some(value))
            } else {
                Err(ObjectError::Other(format!(
                    "no user found for username: {}",
                    username
                )))
            }
        }
        ucl_type::UCL_INT => {
            use std::convert::TryInto;
            let uid: u32 = match src.as_i64().unwrap().try_into() {
                Ok(value) => value,
                Err(e) => {
                    return Err(ObjectError::Other(format!(
                        "could not convert config value to uid: {:#?}",
                        e
                    )))
                }
            };
            if let Some(value) = users::get_user_by_uid(uid) {
                Ok(Some(value))
            } else {
                Err(ObjectError::Other(format!(
                    "no user found for uid: {}",
                    uid
                )))
            }
        }
        _ => Err(ObjectError::Other(
            "user should be either an uid(integer) or a user name(string)".to_owned(),
        )),
    }
}

/// Map an eri config user to an actual user.
fn map_group(src: ObjectRef) -> Result<Option<Group>, ObjectError> {
    if src.is_null() {
        return Ok(None);
    }
    match src.kind() {
        ucl_type::UCL_STRING => {
            let groupname: String = src.as_string().unwrap();
            if let Some(value) = users::get_group_by_name(&groupname) {
                Ok(Some(value))
            } else {
                Err(ObjectError::Other(format!(
                    "no group found for username: {}",
                    groupname
                )))
            }
        }
        ucl_type::UCL_INT => {
            use std::convert::TryInto;
            let gid: u32 = match src.as_i64().unwrap().try_into() {
                Ok(value) => value,
                Err(e) => {
                    return Err(ObjectError::Other(format!(
                        "could not convert config value to gid: {:#?}",
                        e
                    )))
                }
            };
            if let Some(value) = users::get_group_by_gid(gid) {
                Ok(Some(value))
            } else {
                Err(ObjectError::Other(format!(
                    "no group found for gid: {}",
                    gid
                )))
            }
        }
        _ => Err(ObjectError::Other(
            "group should be either a gid(integer) or a group name(string)".to_owned(),
        )),
    }
}

/// The export configuration used for exporting the templates.
#[derive(Clone, Debug, Uclicious)]
pub struct ExportConfig {
    /// The directory where rendered templates should be written to.
    /// By default, it's the current directory.
    #[ucl(default)]
    pub dir: Option<String>,
    /// The user who should own the rendered template.
    /// By default, it's the current user.
    #[ucl(default, map = "map_user")]
    pub user: Option<User>,
    /// The group who should own the rendered template.
    /// By default, it's the current group.
    #[ucl(default, map = "map_group")]
    pub group: Option<Group>,
    /// The permissions that should be applied to a rendered template.
    /// By default, they are the same as the template file.
    #[ucl(default, map = "map_mode")]
    pub permissions: Option<Mode>,
}

impl ExportConfig {
    /// Fill an export config with defaults
    fn fill_defaults(&mut self) {
        if self.dir.is_none() {
            let current_dir_path: PathBuf;
            match std::env::current_dir() {
                Ok(value) => {
                    current_dir_path = value;
                }
                Err(e) => {
                    log::error!("Failed to get the current directory: {:#?}", e);
                    std::process::exit(1);
                }
            }

            if let Some(value) = current_dir_path.to_str() {
                self.dir = Some(value.to_owned());
            } else {
                std::process::exit(1);
            }
        }
    }
}

impl Default for ExportConfig {
    fn default() -> ExportConfig {
        let mut export_config = ExportConfig {
            dir: None,
            user: None,
            group: None,
            permissions: None,
        };
        export_config.fill_defaults();
        export_config
    }
}

/// The eri configuration.
#[derive(Debug, Uclicious)]
pub struct EriConfig {
    /// The export configuration.
    #[ucl(default)]
    pub export: ExportConfig,
    #[ucl(map = "map_namespace")]
    pub namespace: Map<String, Value>,
}

impl EriConfig {
    /// Open the eri configuration.
    /// The configuration is expected to be in the current directory.
    pub fn open() -> Result<Self> {
        if !PathBuf::from("eri.conf").is_file() {
            return Err(anyhow!("eri configuration file(eri.conf) not found"));
        }

        let eri_config_string: String = std::fs::read_to_string("eri.conf")?;

        let mut eri_config_builder = EriConfig::builder()?;
        eri_config_builder
            .add_chunk_full(
                eri_config_string,
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )
            .unwrap();

        match eri_config_builder.build() {
            Ok(mut value) => {
                value.export.fill_defaults();
                Ok(value)
            }
            Err(e) => Err(anyhow!("failed to build eri configuration: {}", e)),
        }
    }

    /// Get the namespaces of the configuration.
    pub fn namespaces(&self) -> Result<Vec<Namespace>> {
        let mut namespaces: Vec<Namespace> = Vec::new();
        for (name, _) in &self.namespace {
            namespaces.push(Namespace::new(
                name,
                &self.export,
                Cow::Borrowed(&self.namespace),
            )?);
        }
        Ok(namespaces)
    }
}
