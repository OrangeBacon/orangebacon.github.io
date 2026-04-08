use std::{
    collections::HashMap,
    error::Error,
    fmt::Write,
    path::{Path, PathBuf},
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use pulldown_cmark_escape::{escape_href, escape_html_body_text};

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

        let mut html = HtmlWriter::new(parser);
        html.run().unwrap();

        let mut metadata: HashMap<_, _> = html
            .metadata
            .lines()
            .flat_map(|l| l.split_once(":"))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();

        metadata.insert("content".to_string(), html.output);
        metadata.insert("intro".to_string(), html.introduction);
        metadata
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

/// HTML writer based on the original from pulldown_cmark, however inlined here
/// to allow modification.
struct HtmlWriter<I> {
    /// Iterator supplying events.
    iter: I,

    /// String to write to.
    output: String,

    /// Whether or not the last write wrote a newline.
    end_newline: bool,

    /// Whether if inside a metadata block
    in_metadata: bool,

    /// The contents of all metadata blocks
    metadata: String,

    /// Is this text the introduction paragraph
    in_intro: bool,

    /// The contents of the introduction paragraph
    introduction: String,
}

impl<'a, I> HtmlWriter<I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I) -> Self {
        Self {
            iter,
            output: String::new(),
            end_newline: true,
            in_metadata: false,
            metadata: String::new(),
            in_intro: true,
            introduction: String::new(),
        }
    }

    /// Writes a buffer, and tracks whether or not a newline was written.
    #[inline]
    fn write(&mut self, s: &str) -> Result<(), Box<dyn Error>> {
        self.output.write_str(s)?;

        if !s.is_empty() {
            self.end_newline = s.ends_with('\n');
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        while let Some(event) = self.iter.next() {
            match event {
                Event::Start(tag) => {
                    self.start_tag(tag)?;
                }
                Event::End(tag) => {
                    self.end_tag(tag)?;
                }
                Event::Text(text) => {
                    if self.in_metadata {
                        self.metadata.push_str(&text);
                    } else {
                        if self.in_intro {
                            self.introduction.push_str(&text);
                        }
                        escape_html_body_text(&mut self.output, &text)?;
                        self.end_newline = text.ends_with('\n');
                    }
                }
                Event::Code(text) => {
                    self.write("<code>")?;
                    escape_html_body_text(&mut self.output, &text)?;
                    self.write("</code>")?;
                }
                Event::SoftBreak => {
                    self.in_intro = false;
                    self.write("\n")?;
                }
                Event::HardBreak => {
                    self.in_intro = false;
                    self.write("<br />\n")?;
                }
                ev => todo!("Impl {ev:?}"),
            }
        }
        Ok(())
    }

    /// Writes the start of an HTML tag.
    fn start_tag(&mut self, tag: Tag<'a>) -> Result<(), Box<dyn Error>> {
        match tag {
            Tag::Paragraph => {
                if self.end_newline {
                    self.write("<p>")
                } else {
                    self.write("\n<p>")
                }
            }
            Tag::Heading { level, .. } => {
                self.in_intro = false;
                if self.end_newline {
                    self.write("<")?;
                } else {
                    self.write("\n<")?;
                }
                write!(&mut self.output, "{}>", level)?;
                Ok(())
            }
            Tag::List(Some(1)) => {
                if self.end_newline {
                    self.write("<ol>\n")
                } else {
                    self.write("\n<ol>\n")
                }
            }
            Tag::List(Some(start)) => {
                if self.end_newline {
                    self.write("<ol start=\"")?;
                } else {
                    self.write("\n<ol start=\"")?;
                }
                write!(&mut self.output, "{}", start)?;
                self.write("\">\n")
            }
            Tag::List(None) => {
                if self.end_newline {
                    self.write("<ul>\n")
                } else {
                    self.write("\n<ul>\n")
                }
            }
            Tag::Item => {
                if self.end_newline {
                    self.write("<li>")
                } else {
                    self.write("\n<li>")
                }
            }
            Tag::Subscript => self.write("<sub>"),
            Tag::Superscript => self.write("<sup>"),
            Tag::Emphasis => self.write("<em>"),
            Tag::Strong => self.write("<strong>"),
            Tag::Strikethrough => self.write("<del>"),
            Tag::Link { dest_url, .. } => {
                self.write("<a href=\"")?;
                escape_href(&mut self.output, &dest_url)?;
                self.write("\">")
            }
            Tag::MetadataBlock(_) => {
                self.in_metadata = true;
                Ok(())
            }
            tag => todo!("impl {tag:?}"),
        }
    }

    fn end_tag(&mut self, tag: TagEnd) -> Result<(), Box<dyn Error>> {
        match tag {
            TagEnd::Paragraph => {
                self.in_intro = false;
                self.write("</p>\n")
            }
            TagEnd::Heading(level) => {
                self.write("</")?;
                write!(&mut self.output, "{}", level)?;
                self.write(">\n")
            }
            TagEnd::Link => self.write("</a>"),
            TagEnd::List(true) => self.write("</ol>\n"),
            TagEnd::List(false) => self.write("</ul>\n"),
            TagEnd::Item => self.write("</li>\n"),
            TagEnd::Emphasis => self.write("</em>"),
            TagEnd::Superscript => self.write("</sup>"),
            TagEnd::Subscript => self.write("</sub>"),
            TagEnd::Strong => self.write("</strong>"),
            TagEnd::Strikethrough => self.write("</del>"),
            TagEnd::FootnoteDefinition => self.write("</div>\n"),
            TagEnd::MetadataBlock(_) => {
                self.in_metadata = false;
                Ok(())
            }
            tag => todo!("impl {tag:?}"),
        }
    }
}
