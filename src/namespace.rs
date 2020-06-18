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
        name: &String,
        export_config: &'a ExportConfig,
        mut data: Cow<'a, Map<String, Value>>,
    ) -> Result<Self> {
        let current_dir_path: PathBuf = match std::env::current_dir() {
            Ok(value) => value,
            Err(e) => panic!("cannot get the current directory: {:?}", e),
        };

        let base_path: PathBuf = current_dir_path.join(name);
        if !base_path.is_dir() {
            return Err(anyhow!("namespace {} does not have a directory", name));
        }

        let additional_eri_conf: PathBuf = base_path.join("eri.conf");
        if additional_eri_conf.exists() {
            if additional_eri_conf.is_file() {
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
                if new_values.len() != 0 {
                    let mut namespace_data: Cow<Map<String, Value>> = match data.to_mut().get(name)
                    {
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
                    data.to_mut().insert(name.clone(), value);
                }
            }
        }

        Ok(Namespace {
            name: name.clone(),
            base_path,
            export_config: Cow::Borrowed(export_config),
            data,
        })
    }

    /// Get the templates in this namespace.
    pub fn templates(&self) -> Result<Vec<Template>> {
        let mut vec: Vec<Template> = Vec::new();

        for file in std::fs::read_dir(&self.base_path)? {
            let file = file.unwrap();
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
                    params.insert(param);
                }
            }
            params
        };
        if params.is_empty() {
            return Ok(());
        }

        let mut file = File::create(self.base_path.join("eri.conf")).unwrap();
        writeln!(file, "# Data file generated by eri at {:?}\n", Local::now())?;
        for param in &params {
            writeln!(file, "{} =", param)?;
        }

        Ok(())
    }

    /// Render all templates inside the namespace.
    pub fn render(&self, handlebars: &mut Handlebars) -> Result<()> {
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
