use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Context {
    pub dirs: Dirs,
}

#[derive(Clone, Debug)]
pub struct Dirs {
    pub content_dir: PathBuf,
    pub template_dir: PathBuf,
    pub stylesheet_dir: PathBuf,
    pub output_dir: PathBuf,
}
