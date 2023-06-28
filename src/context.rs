use std::path::PathBuf;

pub struct Context {
    pub content_dir: PathBuf,
    pub template_dir: PathBuf,
    pub stylesheet_dir: PathBuf,
    pub output_dir: PathBuf,
}
