use std::{collections::HashMap, path::Path};

use crate::file::{FileHandler, SiteEntries};

/// File handler for plain text files, simple file copy with no processing done.
/// Matches all files as a fallback handler.
pub struct TextHandler;

pub const CONTENT_KEY: &str = "content";

impl FileHandler for TextHandler {
    fn matches(&self, _: &Path) -> bool {
        true
    }

    fn metadata(&mut self, _: &Path, content: String) -> HashMap<String, String> {
        HashMap::from([(CONTENT_KEY.to_string(), content)])
    }

    fn output(&self, path: &Path, entries: &SiteEntries) -> Option<String> {
        Some(entries.site_data()[path][CONTENT_KEY].clone())
    }
}
