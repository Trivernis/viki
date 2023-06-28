use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct IndexData {
    /// the default template that is used for rendering
    pub default_template: Option<String>,

    /// files that are included for rendering
    #[serde(default)]
    pub include_files: Vec<String>,

    /// files that are explicitly excluded from rendering
    #[serde(default)]
    pub excluded_files: Vec<String>,
}
