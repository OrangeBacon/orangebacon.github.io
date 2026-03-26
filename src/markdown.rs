use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html::push_html};

use crate::{
    file::{FileHandler, SiteEntries},
    template::ENVIRONMENT,
};

/// File handler for markdown files.  Parses the markdown and puts it into the
/// named template file.
pub struct MarkdownHandler;

impl FileHandler for MarkdownHandler {
    fn matches(&self, path: &Path) -> bool {
        path.extension().map(|e| e == "md").unwrap_or(false)
    }

    fn metadata(&mut self, _: &Path, content: String) -> HashMap<String, String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        let parser = Parser::new_ext(&content, options);

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

        metadata
            .lines()
            .flat_map(|l| l.split_once(":"))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .chain(std::iter::once(("content".to_string(), html)))
            .collect()
    }

    fn output(&self, path: &Path, entries: &SiteEntries) -> Option<String> {
        let metadata = &entries.site_data()[path];

        let env = ENVIRONMENT.lock().unwrap();
        let template = env.get_template(&metadata["template"]).unwrap();
        let output = template.render(metadata).unwrap();
        Some(output)
    }

    fn output_path(&self, path: &Path) -> PathBuf {
        let mut path = path.to_path_buf();
        path.set_extension("html");
        path
    }
}
