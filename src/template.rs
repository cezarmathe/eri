use crate::config::ExportConfig;
use crate::data;

use std::borrow::Cow;
use std::ffi::CString;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use anyhow::Result;

use handlebars::template::Parameter as HandlebarsParameter;
use handlebars::template::Template as HandlebarsTemplate;
use handlebars::template::TemplateElement as HandlebarsTemplateElement;
use handlebars::Handlebars;
use handlebars::Path as HandlebarsPath;

use serde_json::Map;
use serde_json::Value;

/// A template that can be rendered.
#[derive(Debug)]
pub struct Template<'a> {
    pub name: String,
    pub path: PathBuf,
    pub data: &'a Map<String, Value>,
    pub export_config: Cow<'a, ExportConfig>,
}

impl<'a> Template<'a> {
    /// Create a new Template
    pub fn new(
        name: String,
        path: PathBuf,
        data: &'a Map<String, Value>,
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
            name,
            path,
            data,
            export_config,
        })
    }

    /// Register this template in a handlebars object.
    pub fn register(&self, handlebars: &mut Handlebars) -> Result<()> {
        let template_src: String = std::fs::read_to_string(&self.path)?;
        handlebars.register_template_string(&self.name, template_src)?;
        Ok(())
    }

    /// Render this template using the handlebars object.
    pub fn render(&self, handlebars: &mut Handlebars) -> Result<()> {
        log::debug!("Rendering template {}", self.name);
        let template_rendered_string: String = handlebars.render(&self.name, &self.data)?;

        let user: &users::User = self.export_config.user.as_ref().unwrap();
        let group: &users::Group = self.export_config.group.as_ref().unwrap();
        let mode: umask::Mode = self.export_config.permissions.unwrap();

        let path_dir: PathBuf = PathBuf::from(self.export_config.dir.as_ref().unwrap());
        if !path_dir.exists() {
            std::fs::create_dir(&path_dir)?;
            chown(&path_dir, user, group)?;
            let dir_mode: umask::Mode = {
                let mut dir_mode: umask::Mode = mode;
                if !dir_mode.has(umask::USER_EXEC) {
                    dir_mode = dir_mode.with(umask::USER_EXEC);
                }
                if !dir_mode.has(umask::GROUP_EXEC) {
                    dir_mode = dir_mode.with(umask::GROUP_EXEC);
                }
                if !dir_mode.has(umask::OTHERS_EXEC) {
                    dir_mode = dir_mode.with(umask::OTHERS_EXEC);
                }
                dir_mode
            };
            chmod(&path_dir, dir_mode)?;
        } else if !path_dir.is_dir() {
            return Err(anyhow!("export dir already exists"));
        }

        let path_file: PathBuf = path_dir.join(self.filename());
        let mut file: File = File::create(&path_file)?;
        chown(&path_file, user, group)?;
        chmod(&path_file, mode)?;

        write!(file, "{}", template_rendered_string)?;

        Ok(())
    }

    /// Get the parameter list required to render this template.
    pub fn parameter_list(&self, handlebars: &Handlebars) -> Result<Vec<String>> {
        let handlebars_template: &HandlebarsTemplate = match handlebars.get_template(&self.name) {
            Some(value) => value,
            None => return Err(anyhow!("could not find template {}", self.name)),
        };

        let mut parameters: Vec<String> = Vec::new();

        for element in &handlebars_template.elements {
            if let HandlebarsTemplateElement::Expression(expression) = element {
                if let HandlebarsParameter::Path(path) = &expression.name {
                    if let HandlebarsPath::Relative(value) = path {
                        let param: String = value.1.clone();
                        if param.starts_with(&format!("{}.", self.namespace())) {
                            parameters.push(value.1.clone());
                        }
                    }
                }
            }
        }
        Ok(parameters)
    }

    /// Get the name of the namespace of this template.
    pub fn namespace(&self) -> &str {
        let splits: &Vec<&str> = &self.name.split('/').collect();
        splits[0]
    }

    /// Get the file name of this template.
    pub fn filename(&self) -> &str {
        let splits: &Vec<&str> = &self.name.split('/').collect();
        splits[1]
    }
}

fn chown(path: &PathBuf, user: &users::User, group: &users::Group) -> Result<()> {
    let cstr_path: CString = CString::new(path.to_str().unwrap()).unwrap();
    let ret_val: libc::c_int = unsafe { libc::chown(cstr_path.as_ptr(), user.uid(), group.gid()) };
    if ret_val == -1 {
        let errno_val: i32 = errno::errno().into();
        match errno_val {
            libc::EPERM => {
                return Err(anyhow!(
                    "chown: this process lacks permission to make the requested change"
                ))
            }
            libc::EROFS => return Err(anyhow!("chown: the file is on a read-only file system")),
            _ => panic!("chown: unexpected errno: {}", errno_val),
        }
    }
    Ok(())
}

fn chmod(path: &PathBuf, mode: umask::Mode) -> Result<()> {
    let cstr_path: CString = CString::new(path.to_str().unwrap()).unwrap();
    let ret_val: libc::c_int = unsafe { libc::chmod(cstr_path.as_ptr(), mode.into()) };
    if ret_val == -1 {
        let errno_val: i32 = errno::errno().into();
        match errno_val {
            libc::ENOENT => return Err(anyhow!("chmod: the named file doesn’t exist")),
            libc::EPERM => {
                return Err(anyhow!(
                    "chmod: this process does not have permission to change the access permissions of this file"
                ))
            }
            libc::EROFS => {
                return Err(anyhow!("chmod: the file resides on a read-only file system"))
            }
            _ => panic!("chmod: unexpected errno: {}", errno_val),
        }
    }
    Ok(())
}
