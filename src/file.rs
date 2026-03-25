use std::{
    any::Any,
    collections::HashMap,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::OUTPUT_DIR;

/// Collection of all site content, prior to processing.  Contains all files, read
/// from the file system, therefore assumes that the whole site fits in RAM at once.
pub struct SiteEntries {
    content: HashMap<PathBuf, HashMap<String, String>>,
    handlers: Vec<Box<dyn FileHandler>>,
}

/// Trait common for all file type handlers
pub trait FileHandler: Any {
    /// Does the given path apply to this handler
    fn matches(&self, path: &Path) -> bool;

    /// Get the metadata for a given file type.  Will only be called with paths
    /// where `self.matches` returns true.
    fn metadata(&mut self, path: &Path, content: String) -> HashMap<String, String>;

    /// Get the output content for a file, given the metadata for all files in
    /// the site.  Will only be called with paths where `self.matches` returns true.
    /// If None is returned, the output file will not be created.
    fn output(&self, path: &Path, entries: &SiteEntries) -> Option<String>;

    /// Apply modifications to the output path.  By default, returns the input.
    fn output_path(&self, path: &Path) -> PathBuf {
        path.to_path_buf()
    }
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

    /// Get all file handlers
    pub fn handlers(&self) -> impl Iterator<Item = &dyn FileHandler> {
        self.handlers.iter().map(|h| h.as_ref())
    }

    /// Add a file to the site.
    pub fn add(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        let path = path.into();

        for handler in self.handlers.iter_mut() {
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
        for handler in &self.handlers {
            if handler.matches(path) {
                let Some(data) = handler.output(path, self) else {
                    return Ok(());
                };

                let output_path = handler.output_path(&self.output_path(path));

                if let Some(parent) = output_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                let output_file = fs::OpenOptions::new()
                    .truncate(true)
                    .create(true)
                    .write(true)
                    .open(output_path)?;
                let mut buf_write = BufWriter::new(output_file);

                buf_write.write_all(data.as_bytes())?;

                return Ok(());
            }
        }

        Ok(())
    }

    /// Default file path processing:
    /// - If path starts with `./root` return `./`
    /// - Prepend `OUTPUT_DIR`
    fn output_path(&self, path: impl Into<PathBuf>) -> PathBuf {
        let mut path = path.into();

        if let Ok(strip) = path.strip_prefix("./root") {
            path = strip.to_path_buf();
        }

        PathBuf::from(OUTPUT_DIR).join(path)
    }
}
