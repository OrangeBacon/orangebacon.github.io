use std::{
    collections::HashMap,
    path::{Component, Path},
    sync::{LazyLock, Mutex},
};

use minijinja::Environment;

use crate::{
    file::{FileHandler, SiteEntries},
    plain_text::CONTENT_KEY,
};

pub static ENVIRONMENT: LazyLock<Mutex<Environment<'static>>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::None);
    env.add_function("remove_extension", remove_extension);
    Mutex::new(env)
});

/// File handler for the base template files, parses them and ensures they can
/// be accessed.  Does not manage substitution into the templates, that depends
/// upon the output file handlers.
pub struct TemplateHandler;

impl FileHandler for TemplateHandler {
    fn matches(&self, path: &Path) -> bool {
        let mut components = path.components();

        if let Some(Component::Normal(filter_dir)) = components.nth(1)
            && filter_dir == "templates"
        {
            return true;
        }

        false
    }

    fn metadata(&mut self, path: &Path, content: String) -> HashMap<String, String> {
        ENVIRONMENT
            .lock()
            .unwrap()
            .add_template_owned(path.to_string_lossy().to_string(), content.clone())
            .unwrap();

        HashMap::from([(CONTENT_KEY.to_string(), content)])
    }

    fn output(&self, _: &Path, _: &SiteEntries) -> Option<String> {
        None
    }
}

/// Helper to remove the file extension from a path
fn remove_extension(path: String) -> String {
    let mut buf = Path::new(&path).to_path_buf();
    buf.set_extension("");

    buf.to_string_lossy().to_string()
}
