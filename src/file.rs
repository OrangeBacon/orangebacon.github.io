use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use crate::OUTPUT_DIR;

/// Collection of all site content, prior to processing.  Contains all files, read
/// from the file system, therefore assumes that the whole site fits in RAM at once.
pub struct SiteEntries {
    content: HashMap<PathBuf, HashMap<String, String>>,
    handlers: Vec<Box<dyn FileHandler>>,
}

/// Trait common for all file type handlers
pub trait FileHandler {
    /// Does the given path apply to this handler
    fn matches(&self, path: &Path) -> bool;

    /// Get the metadata for a given file type.  Will only be called with paths
    /// where `self.matches` returns true.
    fn metadata(&self, path: &Path, content: String) -> HashMap<String, String>;

    /// Get the output content for a file, given the metadata for all files in
    /// the site.  Will only be called with paths where `self.matches` returns true.
    fn output(&self, path: &Path, entries: &SiteEntries) -> String;
}

impl SiteEntries {
    /// Create a new site
    pub fn new() -> Self {
        Self {
            content: HashMap::new(),
            handlers: vec![],
        }
    }

    /// Get all data for the site
    pub fn site_data(&self) -> &HashMap<PathBuf, HashMap<String, String>> {
        &self.content
    }

    /// Add a handler for a file type.  The order they are checked in is equal to
    /// the order they are added, so add more specific ones first.
    pub fn handler<H: FileHandler + 'static>(&mut self, handler: H) {
        self.handlers.push(Box::new(handler));
    }

    /// Add a file to the site.
    pub fn add(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        let path = path.into();

        for handler in &self.handlers {
            if handler.matches(&path) {
                let data = handler.metadata(&path, content.into());
                self.content.insert(path, data);
                return;
            }
        }
    }

    /// Process all files in the site and write the output to the filesystem.
    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        for path in self.content.keys() {
            self.process_file(path)?;
        }

        Ok(())
    }

    /// Handle all file processing
    pub fn process_file(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        // filter out template files
        let components: Vec<_> = path.components().collect();
        if let Some(Component::Normal(filter_dir)) = components.get(1) {
            let name = filter_dir.to_string_lossy();
            if name == "templates" {
                return Ok(());
            }
        }

        let output_path = self.output_path(path);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let output_file = fs::OpenOptions::new()
            .truncate(true)
            .create(true)
            .write(true)
            .open(output_path)?;
        let mut buf_write = BufWriter::new(output_file);

        for handler in &self.handlers {
            if handler.matches(path) {
                let data = handler.output(path, self);

                buf_write.write_all(data.as_bytes())?;

                return Ok(());
            }
        }

        Ok(())
    }

    /// Get the path of a file, following processing
    /// - If path starts with `./root` return `./`
    /// - If the file extension is ".md", replace it with ".html"
    /// - Prepend `OUTPUT_DIR`
    fn output_path(&self, path: &Path) -> PathBuf {
        let mut path = path.to_path_buf();

        if let Ok(strip) = path.strip_prefix("./root") {
            path = strip.to_path_buf();
        }

        if path.extension().map(|e| e == "md").unwrap_or(false) {
            path.set_extension("html");
        }

        PathBuf::from(OUTPUT_DIR).join(path)
    }
}
