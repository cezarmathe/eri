use crate::config::ExportConfig;
use crate::data;
use crate::template::*;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use chrono::offset::Local;

use handlebars::Handlebars;

use serde_json::Map;
use serde_json::Value;

use uclicious::Parser;
use uclicious::Priority;
use uclicious::DEFAULT_DUPLICATE_STRATEGY;

/// General representation of a namespace of templates.
#[derive(Debug)]
pub struct Namespace<'a> {
    pub name: String,
    pub base_path: PathBuf,
    pub export_config: Cow<'a, ExportConfig>,
    pub data: Cow<'a, Map<String, Value>>,
}

impl<'a> Namespace<'a> {
    /// Create a new namespace.
    pub fn new(
        name: &str,
        export_config: &'a ExportConfig,
        mut data: Cow<'a, Map<String, Value>>,
    ) -> Result<Self> {
        let current_dir_path: PathBuf = match std::env::current_dir() {
            Ok(value) => value,
            Err(e) => {
                log::error!("Cannot get the current directory: {:#?}", e);
                std::process::exit(1);
            }
        };

        let base_path: PathBuf = current_dir_path.join(name);
        if !base_path.is_dir() {
            return Err(anyhow!("namespace {} does not have a directory", name));
        }

        let additional_eri_conf: PathBuf = base_path.join("eri.conf");
        if additional_eri_conf.exists() && additional_eri_conf.is_file() {
            let eri_config_string: String = std::fs::read_to_string(additional_eri_conf)?;
            let mut parser: Parser = Parser::default();
            parser.add_chunk_full(
                eri_config_string,
                Priority::default(),
                DEFAULT_DUPLICATE_STRATEGY,
            )?;
            let mut new_values: Map<String, Value> = Map::new();
            for item in parser.get_object()?.iter() {
                let item_key = item.key().unwrap();
                match data::object_ref_to_value(item) {
                    Ok(value) => new_values.insert(item_key, value),
                    Err(e) => return Err(e),
                };
            }
            if !new_values.is_empty() {
                let mut namespace_data: Cow<Map<String, Value>> = match data.to_mut().get(name) {
                    Some(value) => {
                        if let Value::Object(obj) = value {
                            Cow::Borrowed(obj)
                        } else {
                            panic!("something messed up the data")
                        }
                    }
                    None => Cow::Owned(Map::new()),
                };
                namespace_data.to_mut().append(&mut new_values);
                let value: Value = Value::Object(namespace_data.into_owned());
                data.to_mut().insert(name.to_owned(), value);
            }
        }

        Ok(Namespace {
            name: name.to_owned(),
            base_path,
            export_config: Cow::Borrowed(export_config),
            data,
        })
    }

    /// Get the templates in this namespace.
    pub fn templates(&self) -> Result<Vec<Template>> {
        let mut vec: Vec<Template> = Vec::new();

        for file in std::fs::read_dir(&self.base_path)? {
            let file = file?;
            let file_path: PathBuf = file.path();
            if file_path.ends_with("eri.conf") {
                continue;
            }
            let file_name: String = if let Some(os_str) = file_path.file_name() {
                if let Some(value) = os_str.to_str() {
                    value.to_owned()
                } else {
                    return Err(anyhow!("failed to get convert the file name into a string"));
                }
            } else {
                return Err(anyhow!(
                    "failed to get the file name of the template at {:?}",
                    file.path()
                ));
            };
            let _template: Template = Template::new(
                format!("{}/{}", &self.name, file_name),
                file.path(),
                &self.data,
                std::borrow::Cow::Borrowed(&self.export_config),
            )?;
            vec.push(_template);
        }

        Ok(vec)
    }

    /// Generate a data file required by this namespace.
    pub fn gen_data_file(&self, handlebars: &mut Handlebars) -> Result<()> {
        let templates: Vec<Template> = self.templates()?;
        for template in &templates {
            template.register(handlebars)?;
        }

        let params: BTreeSet<String> = {
            let mut params: BTreeSet<String> = BTreeSet::new();
            for template in templates {
                for param in template.parameter_list(handlebars)? {
                    let mut param_parts: Vec<&str> = param.split('.').collect();
                    param_parts.remove(0);
                    params.insert(param_parts.join("."));
                }
            }
            params
        };
        if params.is_empty() {
            return Ok(());
        }

        let data_file_path: PathBuf = self.base_path.join("eri.conf");
        if data_file_path.exists() && data_file_path.is_file() {
            let data_file_backup_path: PathBuf = self
                .base_path
                .join(format!("eri.conf.bk_{:?}", Local::now()));
            log::info!(
                "Data file for namespace {} already exists at {:?}, making a backup at {:?}",
                self.name,
                data_file_path,
                data_file_backup_path
            );
            match std::fs::copy(&data_file_path, data_file_backup_path) {
                Ok(_) => log::debug!("Backup successful"),
                Err(e) => {
                    return Err(anyhow!("Failed to make a backup, not proceeding: {:#?}", e));
                }
            }
        }

        let mut file = match File::create(&data_file_path) {
            Ok(value) => value,
            Err(e) => {
                return Err(anyhow!(
                    "could not create data file for namespace {}: {:#?}",
                    self.name,
                    e
                ))
            }
        };
        writeln!(
            file,
            "# Data file generated by eri {} at {:?}\n",
            crate::ERI_VERSION,
            Local::now()
        )?;
        for param in &params {
            writeln!(file, "{} =", param)?;
        }

        log::info!(
            "Data file for namespace {} generated at {:?}",
            self.name,
            data_file_path
        );

        Ok(())
    }

    /// Render all templates inside the namespace.
    pub fn render(&self, handlebars: &mut Handlebars) -> Result<()> {
        log::info!("Rendering namespace {}", self.name);
        let templates: Vec<Template> = self.templates()?;
        for template in &templates {
            template.register(handlebars)?;
        }
        for template in &templates {
            template.render(handlebars)?;
        }
        Ok(())
    }
}
