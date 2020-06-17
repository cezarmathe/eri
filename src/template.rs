use crate::config::ExportConfig;
use crate::data;

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::path::PathBuf;

use anyhow::Result;

use handlebars::Handlebars;

use serde_json::Value;

/// Data used by a template.
pub type TemplateData<'a> = BTreeMap<String, &'a BTreeMap<String, Value>>;

/// A template that can be rendered.
#[derive(Debug)]
pub struct Template<'a> {
    pub name: String,
    pub path: PathBuf,
    pub data: TemplateData<'a>,
    pub export_config: Cow<'a, ExportConfig>,
}

impl<'a> Template<'a> {
    /// Create a new Template
    pub fn new(
        path: PathBuf,
        data: TemplateData<'a>,
        mut export_config: Cow<'a, ExportConfig>,
    ) -> Result<Self> {
        if path.is_dir() {
            panic!("template is not supposed to be created with a directory path");
        }

        if export_config.user.is_none() {
            export_config.to_mut().user = Some(data::get_user(&path)?);
        }
        if export_config.group.is_none() {
            export_config.to_mut().group = Some(data::get_group(&path)?);
        }
        if export_config.permissions.is_none() {
            export_config.to_mut().permissions = Some(data::get_permissions(&path)?);
        }

        Ok(Self {
            name: path.file_name().unwrap().to_str().unwrap().to_owned(),
            path,
            data,
            export_config,
        })
    }

    pub fn render(&self, handlebars: &mut Handlebars) -> Result<()> {
        let template_src: String = std::fs::read_to_string(&self.path)?;
        handlebars.register_template_string(&self.name, template_src)?;
        // println!("\nTemplate: {:?}", handlebars.get_template("vault.hcl").unwrap().elements);

        let template_rendered_string: String = handlebars.render(&self.name, &self.data)?;
        println!("\nApplied template:\n{}", template_rendered_string);

        Ok(())
    }
}
