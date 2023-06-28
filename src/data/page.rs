use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum Page {
    Data(PageData),
    Content(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PageData {
    /// Metadata for this page
    #[serde(default)]
    pub metadata: PageMetadata,

    /// remaining data of this page
    /// passed to the templates when rendering
    #[serde(flatten)]
    pub data: toml::Value,
}

#[derive(Default, Clone, Debug, Deserialize, Serialize)]
pub struct PageMetadata {
    /// template used to render this page
    pub template: Option<String>,
}
