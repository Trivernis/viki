use std::collections::HashMap;

use globset::Glob;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct IndexData {
    /// the default template that is used for rendering
    pub default_template: Option<String>,

    /// files that are included for rendering
    pub include_files: Vec<Glob>,

    /// files that are explicitly excluded from rendering
    pub excluded_files: Vec<Glob>,

    /// File paths with templates used to rendering them
    pub templates: HashMap<Glob, String>,
}
