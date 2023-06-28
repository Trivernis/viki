use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct PageMetadata {
    /// template used to render this page
    pub template: Option<String>,

    /// remaining data of this page
    /// passed to the templates when rendering
    #[serde(flatten)]
    pub data: toml::Value,
}
