use crate::config::ExportConfig;
use crate::namespace::NamespaceData;

use std::borrow::Cow;
use std::collections::BTreeMap;

/// Data used by a template.
pub type TemplateData<'a> = BTreeMap<String, &'a NamespaceData>;

/// A template that can be rendered.
pub struct Template<'a> {
    pub name: String,
    pub data: TemplateData<'a>,
    pub export_config: Cow<'a, ExportConfig>,
}
