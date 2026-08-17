use std::{
    collections::HashMap,
    path::{Component, Path},
    sync::{LazyLock, Mutex},
};

use jiff::civil::Date;
use minijinja::Environment;

use crate::{
    file::{FileHandler, SiteEntries},
    plain_text::CONTENT_KEY,
};

pub static ENVIRONMENT: LazyLock<Mutex<Environment<'static>>> = LazyLock::new(|| {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    env.add_filter("remove_extension", remove_extension);
    env.add_filter("remove_dot_slash", remove_dot_slash);
    env.add_filter("date_to_rss", date_to_rss);
    env.add_filter("date_to_rfc3339", date_to_rfc3339);
    env.add_function("file", file);
    Mutex::new(env)
});

/// File handler for the base template files, parses them and ensures they can
/// be accessed.  Does not manage substitution into the templates, that depends
/// upon the output file handlers.
pub struct TemplateHandler;

impl FileHandler for TemplateHandler {
    fn matches(&self, path: &Path) -> bool {
        let components: Vec<_> = path.components().collect();

        if let &[.., Component::Normal(filter_dir), _] = components.as_slice()
            && filter_dir == "templates"
        {
            return true;
        }
        if let &[.., Component::Normal(name)] = components.as_slice()
            && name.to_str().is_some_and(|n| n.contains("tmpl"))
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
    if buf.file_stem().is_some_and(|s| s == "index") {
        buf.pop();
    } else {
        buf.set_extension("");
    }

    buf.to_string_lossy().to_string()
}

/// Helper to remove the leading "./" from a path
fn remove_dot_slash(path: String) -> String {
    path.strip_prefix("./")
        .map(|s| s.to_owned())
        .unwrap_or(path)
}

/// Helper to format dates correctly for RSS
fn date_to_rss(date: String) -> String {
    Date::strptime("%Y-%m-%d", date)
        .unwrap()
        .strftime("%a, %d %b %Y 00:00:00 GMT")
        .to_string()
}

/// Helper to format dates correctly for Atom feeds
fn date_to_rfc3339(date: String) -> String {
    Date::strptime("%Y-%m-%d", date)
        .unwrap()
        .strftime("%Y-%m-%dT00:00:00Z")
        .to_string()
}

/// Read a file from within a template
fn file(path: String) -> String {
    std::fs::read_to_string(path).unwrap()
}
