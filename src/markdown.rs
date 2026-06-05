use std::{
    collections::HashMap,
    error::Error,
    fmt::Write,
    path::{Path, PathBuf},
};

use pulldown_cmark::{CodeBlockKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use pulldown_cmark_escape::{escape_href, escape_html, escape_html_body_text};

use crate::{
    file::{FileHandler, SiteEntries},
    highlight::SyntaxHighlighter,
    template::ENVIRONMENT,
};

/// File handler for markdown files.  Parses the markdown and puts it into the
/// named template file.
pub struct MarkdownHandler {
    highlighter: SyntaxHighlighter,
}

impl MarkdownHandler {
    pub fn new() -> Self {
        Self {
            highlighter: SyntaxHighlighter::new().unwrap(),
        }
    }
}

impl FileHandler for MarkdownHandler {
    fn matches(&self, path: &Path) -> bool {
        path.extension().map(|e| e == "md").unwrap_or(false)
    }

    fn metadata(&mut self, _: &Path, content: String) -> HashMap<String, String> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_YAML_STYLE_METADATA_BLOCKS);
        options.insert(Options::ENABLE_FOOTNOTES);
        let parser = Parser::new_ext(&content, options);

        let mut html = HtmlWriter::new(parser, self);
        html.run().unwrap();

        let mut metadata: HashMap<_, _> = html
            .metadata
            .lines()
            .flat_map(|l| l.split_once(":"))
            .map(|(k, v)| (k.trim().to_string(), v.trim().to_string()))
            .collect();

        metadata.insert("content".to_string(), html.output.as_str().to_string());

        if !metadata.contains_key("intro") {
            metadata.insert("intro".to_string(), html.introduction);
        }
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
struct HtmlWriter<'a, I> {
    /// Iterator supplying events.
    iter: I,

    /// String to write to.
    output: MultiString,

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

    /// Map footnote identifier to footnote number
    footnote_links: HashMap<String, usize>,

    /// Map footnote number to definition content
    footnote_defs: HashMap<usize, (String, String)>,

    /// If in a code block, this contains the language identifier, otherwise None
    in_syntax: Option<String>,

    /// Contents of the current code block
    code_block: String,

    highlighter: &'a mut SyntaxHighlighter,
}

impl<'a, I> HtmlWriter<'a, I>
where
    I: Iterator<Item = Event<'a>>,
{
    fn new(iter: I, markdown: &'a mut MarkdownHandler) -> Self {
        Self {
            iter,
            output: MultiString::default(),
            end_newline: true,
            in_metadata: false,
            metadata: String::new(),
            in_intro: true,
            introduction: String::new(),
            footnote_links: HashMap::new(),
            footnote_defs: HashMap::new(),
            in_syntax: None,
            code_block: String::new(),
            highlighter: &mut markdown.highlighter,
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
                    } else if self.in_syntax.is_some() {
                        self.code_block.push_str(&text);
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
                Event::FootnoteReference(name) => {
                    let len = self.footnote_links.len() + 1;
                    self.write("<a href='#fn-")?;
                    escape_html(&mut self.output, &name)?;
                    self.write("' role='doc-noteref' class='footnote-reference'>")?;
                    let number = *self.footnote_links.entry(name.to_string()).or_insert(len);
                    write!(&mut self.output, "{}", number)?;
                    self.write("</a>")?;
                }
                ev => todo!("Impl {ev:?}"),
            }
        }

        let mut notes: Vec<_> = std::mem::take(&mut self.footnote_defs)
            .into_iter()
            .collect();
        notes.sort_by_key(|&(n, _)| n);

        if !notes.is_empty() {
            if self.end_newline {
                self.write("<hr aria-hidden='true'/>")?;
            } else {
                self.write("\n<hr aria-hidden='true'/>")?;
            }
        }

        for (_, (name, note)) in notes {
            if self.end_newline {
                self.write("<div class=\"footnote-definition\" id=\"fn-")?;
            } else {
                self.write("\n<div class=\"footnote-definition\" id=\"fn-")?;
            }
            escape_html(&mut self.output, &name)?;
            self.write("\"><span class=\"footnote-definition-label\">")?;
            let len = self.footnote_links.len() + 1;
            let number = *self.footnote_links.entry(name.clone()).or_insert(len);
            write!(&mut self.output, "{}: ", number)?;
            self.write("</span>")?;
            self.write(&note)?;
            self.write("</div>\n")?;
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
                write!(&mut self.output, "{}>", reduce_header(level))?;
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
            Tag::FootnoteDefinition(name) => {
                self.output.push(name.to_string());
                Ok(())
            }
            Tag::CodeBlock(kind) => {
                self.in_intro = false;
                self.in_syntax = Some(match kind {
                    CodeBlockKind::Indented => String::new(),
                    CodeBlockKind::Fenced(s) => s.to_string(),
                });
                self.code_block = String::new();
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
                write!(&mut self.output, "{}", reduce_header(level))?;
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
            TagEnd::MetadataBlock(_) => {
                self.in_metadata = false;
                Ok(())
            }
            TagEnd::FootnoteDefinition => {
                let (name, note) = self.output.pop();
                let len = self.footnote_links.len() + 1;
                let number = *self.footnote_links.entry(name.clone()).or_insert(len);
                self.footnote_defs.insert(number, (name, note));
                Ok(())
            }
            TagEnd::CodeBlock => self.highlighted_code(),
            tag => todo!("impl {tag:?}"),
        }
    }

    fn highlighted_code(&mut self) -> Result<(), Box<dyn Error>> {
        let lang = self.in_syntax.take().unwrap_or_default();

        if lang == "pikchr" {
            return self.diagram();
        }

        let out = self.highlighter.highlight(&lang, &self.code_block)?;
        writeln!(self.output, "{out}")?;
        self.end_newline = true;

        Ok(())
    }

    fn diagram(&mut self) -> Result<(), Box<dyn Error>> {
        let dia = pikchr::Pikchr::render(
            &self.code_block,
            None,
            *pikchr::PikchrFlags::default().use_dark_mode(),
        )?;

        self.write(&dia)
    }
}

/// Change heading levels down by 1, e.g. h1 -> h2.  Used because the title is
/// meant to be h1, so h1 in the markdown should render as an h2.
fn reduce_header(h: HeadingLevel) -> HeadingLevel {
    match h {
        HeadingLevel::H1 => HeadingLevel::H2,
        HeadingLevel::H2 => HeadingLevel::H3,
        HeadingLevel::H3 => HeadingLevel::H4,
        HeadingLevel::H4 => HeadingLevel::H5,
        HeadingLevel::H5 => HeadingLevel::H6,
        HeadingLevel::H6 => HeadingLevel::H6,
    }
}

#[derive(Default)]
struct MultiString {
    base: String,
    other: Vec<(String, String)>,
}

impl MultiString {
    /// Get the string
    pub fn as_str(&self) -> &str {
        self.other.last().map(|s| &s.1).unwrap_or(&self.base)
    }

    /// Add a layer to the string, with its name.
    pub fn push(&mut self, name: String) {
        self.other.push((name, String::new()));
    }

    /// Remove the top most layer and return it.  The name of the layer is the
    /// first string in the tuple.
    pub fn pop(&mut self) -> (String, String) {
        self.other.pop().unwrap_or_default()
    }
}
impl std::fmt::Write for MultiString {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.other
            .last_mut()
            .map(|s| &mut s.1)
            .unwrap_or(&mut self.base)
            .push_str(s);

        Ok(())
    }
}
impl pulldown_cmark_escape::StrWrite for MultiString {
    type Error = Box<dyn Error>;

    fn write_str(&mut self, s: &str) -> Result<(), Self::Error> {
        std::fmt::Write::write_str(self, s)?;

        Ok(())
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments) -> Result<(), Self::Error> {
        std::fmt::Write::write_fmt(self, args)?;

        Ok(())
    }
}
