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

/// Map permissions mode.
fn map_mode(src: ObjectRef) -> Result<Option<Mode>, ObjectError> {
    log::trace!("mapping ucl object {:?} to permissions mode", src);
    if src.is_null() {
        log::trace!("ucl object is null, returning missing permissions");
        return Ok(None);
    }
    log::trace!("checking if ucl object is an integer");
    if !src.is_integer() {
        log::trace!("ucl object is not an integer, returning an error");
        return Err(ObjectError::WrongType {
            key: "permissions".to_owned(),
            actual_type: src.kind(),
            wanted_type: ucl_type::UCL_INT,
        });
    }
    log::trace!("ucl object is an integer");

    log::trace!("converting ucl object to an integer");
    use uclicious::TryInto;
    let val: u32 = src.try_into()?;
    log::trace!("converted ucl object to an integer");

    log::trace!("converting integer from base 8 to base 10");
    let user_value: u32 = val / 100;
    let group_value: u32 = val % 100 / 10;
    let all_value: u32 = val % 10;
    log::trace!("converted integer from base 8 to base 10");

    log::trace!("returning permissions mode");
    Ok(Some(Mode::from(
        user_value * 64 + group_value * 8 + all_value,
    )))
}

/// Map the eri config namespaces from ucl.
pub fn map_namespace(src: ObjectRef) -> Result<Map<String, Value>, ObjectError> {
    log::trace!("mapping namespaces from configuration");
    let mut result: Map<String, Value> = Map::new();

    for item in src.iter() {
        log::trace!("mapping item {:?}", item);
        let item_key = item.key().unwrap();
        log::trace!("namespace key is {}", item_key);
        match data::object_ref_to_value(item) {
            Ok(value) => {
                log::trace!("converted ucl object to value");
                result.insert(item_key, value)
            },
            Err(e) => {
                log::trace!("failed to convert ucl object to value");
                return Err(ObjectError::Other(e.to_string()));
            },
        };
    }

    log::trace!("returning namespace map: {:?}", result);
    Ok(result)
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
    #[ucl(default)]
    pub user: Option<String>,
    /// The group who should own the rendered template.
    /// By default, it's the current group.
    #[ucl(default)]
    pub group: Option<String>,
    /// The permissions that should be applied to a rendered template.
    /// By default, they are the same as the template file.
    #[ucl(default, map = "map_mode")]
    pub permissions: Option<Mode>,
}

impl ExportConfig {
    /// Fill an export config with defaults
    fn fill_defaults(&mut self) {
        log::trace!("filling export configuration with the defaults");
        if self.dir.is_none() {
            log::trace!("filling the directory for the export configuration with the default value");
            let current_dir_path: PathBuf;
            match std::env::current_dir() {
                Ok(value) => {
                    current_dir_path = value;
                    log::trace!("current dir path: {:?}", current_dir_path);
                },
                Err(e) => {
                    log::error!("cannot get the current directory: {:#?}", e);
                    std::process::exit(1);
                },
            }

            log::trace!("converting current dir path to string");
            if let Some(value) = current_dir_path.to_str() {
                log::trace!("converted current dir path to string");
                self.dir = Some(value.to_owned());
            } else {
                log::error!("failed to convert current directory path into a string");
                std::process::exit(1);
            }
        }
    }
}

impl Default for ExportConfig {
    fn default() -> ExportConfig {
        log::trace!("requested default export config");
        let mut export_config = ExportConfig {
            dir: None,
            user: None,
            group: None,
            permissions: None,
        };
        export_config.fill_defaults();
        log::trace!("default export config: {:?}", export_config);
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
