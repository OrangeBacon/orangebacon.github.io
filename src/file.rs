use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html::push_html};

use crate::OUTPUT_DIR;

/// Collection of all site content, prior to processing.  Contains all files, read
/// from the file system, therefore assumes that the whole site fits in RAM at once.
pub struct SiteEntries {
    content: HashMap<PathBuf, String>,
}

impl SiteEntries {
    /// Create a new site
    pub fn new() -> Self {
        Self {
            content: HashMap::new(),
        }
    }

    /// Add a file to the site.
    pub fn add(&mut self, path: impl Into<PathBuf>, content: impl Into<String>) {
        self.content.insert(path.into(), content.into());
    }

    /// Process all files in the site and write the output to the filesystem.
    pub fn process(&self) -> Result<(), Box<dyn Error>> {
        for (path, content) in &self.content {
            self.process_file(path, content)?;
        }

        Ok(())
    }

    /// Handle all file processing
    pub fn process_file(&self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
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

        if path.extension().map(|e| e == "md").unwrap_or(false) {
            self.process_markdown(content, buf_write)?;
        } else {
            buf_write.write_all(content.as_bytes())?;
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

    /// Process a markdown file
    fn process_markdown(
        &self,
        input: &str,
        mut writer: impl std::io::Write,
    ) -> Result<(), Box<dyn Error>> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        let parser = Parser::new_ext(input, options);

        // extract the metadata from the parser
        let mut metadata = String::new();
        let mut in_meta = false;
        let parser = parser.filter(|ev| match ev {
            Event::Start(Tag::MetadataBlock(_)) => {
                in_meta = true;
                false
            }
            Event::End(TagEnd::MetadataBlock(_)) => {
                in_meta = false;
                false
            }
            Event::Text(txt) if in_meta => {
                metadata.push_str(txt);
                false
            }
            _ => true,
        });

        let mut html = String::new();
        push_html(&mut html, parser);

        let metadata: HashMap<_, _> = metadata
            .lines()
            .flat_map(|l| l.split_once(":"))
            .map(|(k, v)| (k.trim(), v.trim()))
            .chain(std::iter::once(("content", html.as_str())))
            .collect();

        // get the template and replace the directives in it
        let template = &self.content[Path::new(&metadata["template"])];

        let mut output = template.clone();
        for (key, value) in metadata {
            output = output.replace(&format!("{{% {key} %}}"), value);
        }

        writer.write_all(output.as_bytes())?;

        Ok(())
    }
}
