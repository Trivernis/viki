use std::{collections::HashMap, path::PathBuf};

use async_walkdir::WalkDir;
use futures::StreamExt;
use miette::{IntoDiagnostic, Result};

pub struct Stylesheets {
    pub default_style: Option<PathBuf>,
    pub page_styles: HashMap<String, PathBuf>,
}

pub async fn load_stylesheets(base_dir: &PathBuf) -> Result<Stylesheets> {
    let mut entries = WalkDir::new(base_dir);
    let mut page_styles = HashMap::new();

    while let Some(res) = entries.next().await {
        match res {
            Ok(entry) => {
                let entry_path = entry.path();
                if entry_path.is_file() {
                    if let Some(file_name) = entry_path.file_stem() {
                        let file_name = file_name.to_string_lossy().into_owned();
                        page_styles.insert(file_name, entry_path.to_owned());
                    }
                }
            }
            Err(e) => return Err(e).into_diagnostic(),
        }
    }

    let mut default_style = None;
    for name in ["style", "default", "stylesheet", "index"] {
        if let Some(style) = page_styles.remove(name) {
            default_style = Some(style);
            break;
        }
    }

    Ok(Stylesheets {
        default_style,
        page_styles,
    })
}
