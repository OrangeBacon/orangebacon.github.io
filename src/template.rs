use std::{
    collections::HashMap,
    path::{Component, Path},
};

use minijinja::Environment;

use crate::{
    file::{FileHandler, SiteEntries},
    plain_text::CONTENT_KEY,
};

/// File handler for the base template files, parses them and ensures they can
/// be accessed.  Does not manage substitution into the templates, that depends
/// upon the output file handlers.
#[derive(Default)]
pub struct TemplateHandler {
    env: Environment<'static>,
}

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
        self.env
            .add_template_owned(path.to_string_lossy().to_string(), content.clone());

        HashMap::from([(CONTENT_KEY.to_string(), content)])
    }

    fn output(&self, _: &Path, _: &SiteEntries) -> Option<String> {
        None
    }
}
