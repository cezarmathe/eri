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
pub fn map_namespace(src: ObjectRef) -> Result<Map<String, Value>, ObjectError> {
    let mut result: Map<String, Value> = Map::new();

    for item in src.iter() {
        let item_key = item.key().unwrap();
        match data::object_ref_to_value(item) {
            Ok(value) => result.insert(item_key, value),
            Err(e) => return Err(ObjectError::Other(e.to_string())),
        };
    }

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
        if self.dir.is_none() {
            let current_dir_path: PathBuf;
            match std::env::current_dir() {
                Ok(value) => current_dir_path = value,
                Err(e) => panic!("cannot get the current directory: {:?}", e),
            }

            if let Some(value) = current_dir_path.to_str() {
                self.dir = Some(value.to_owned());
            } else {
                panic!("failed to convert current directory path into a string");
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
