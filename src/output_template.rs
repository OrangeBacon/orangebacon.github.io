use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use minijinja::context;

use crate::{
    file::{FileHandler, SiteEntries},
    plain_text::CONTENT_KEY,
    template::ENVIRONMENT,
};

/// File handler for plain text files, simple file copy with no processing done.
/// Matches all files as a fallback handler.
pub struct OutputTemplate;

impl FileHandler for OutputTemplate {
    fn matches(&self, path: &Path) -> bool {
        path.extension().map(|e| e == "jinja").unwrap_or(false)
    }

    fn metadata(&mut self, path: &Path, content: String) -> HashMap<String, String> {
        ENVIRONMENT
            .lock()
            .unwrap()
            .add_template_owned(path.to_string_lossy().to_string(), content.clone())
            .unwrap();

        HashMap::from([(CONTENT_KEY.to_string(), content)])
    }

    fn output(&self, path: &Path, entries: &SiteEntries) -> Option<String> {
        let metadata = entries.site_data();

        let env = ENVIRONMENT.lock().unwrap();
        let template = env.get_template(&path.to_string_lossy()).unwrap();
        let output = template
            .render(context! {
                posts => metadata,
                path,
            })
            .unwrap();
        Some(output)
    }

    fn output_path(&self, path: &Path) -> PathBuf {
        let mut path = path.to_path_buf();
        path.set_extension("");
        path
    }
}
