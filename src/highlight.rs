mod unknown;

use std::{collections::HashMap, error::Error, fmt::Display};

use pulldown_cmark_escape::{FmtWriter, escape_html_body_text};

/// The description of a single source code snippet
#[derive(Debug, Clone)]
struct SourceDescription<'a> {
    /// The language name
    language: String,

    /// Properties `a=5` to be passed to the highlighter
    properties: HashMap<&'a str, &'a str>,
}

/// A highlighted source file
struct HighlightedSource {
    /// All data to be displayed
    lines: Vec<HighlightedLine>,
}

/// A single line within a highlighted source code file.
struct HighlightedLine {
    /// Highlights that apply to the whole line
    kind: HighlightedLineKind,

    /// Individual elements of the source line
    content: Vec<(HighlightScope, String)>,
}

/// Highlights for a whole line of code
enum HighlightedLineKind {
    None,
    Highlighted,
    DiffAddition,
    DiffRemoval,
}

/// All possible highlighting region kinds, based on those from Highlight.js
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HighlightScope {
    /// No specific highlighting scope (whitespace, etc.)
    None,

    /// keyword in a regular Algol-style language
    Keyword,

    // built-in or library object (constant, class, function)
    BuiltIn,

    // data type (in a language with syntactically significant types) (string, int, array, etc.)
    Type,

    /// special identifier for a built-in value (true, false, null, etc.)
    Literal,

    /// number, including units and modifiers, if any.
    Number,

    /// operators: +, -, >>, |, ==
    Operator,

    /// aux. punctuation that should be subtly highlighted (parentheses, brackets, etc.)
    Punctuation,

    /// object property obj.prop1.prop2.value
    Property,

    /// literal regular expression
    Regexp,

    /// literal string, character
    String,

    /// an escape character such as \n
    Escape,

    /// symbolic constant, interned string, goto label
    Symbol,

    // variables
    Variable,

    /// variable that is a constant value, ie MAX_FILES
    Constant,

    // name of a class (interface, trait, module, etc)
    Class,

    // name of a function
    Function,

    /// comments
    Comment,

    /// documentation markup within comments, e.g. @params
    DocTag,

    /// flags, modifiers, annotations, processing instructions, preprocessor directives, etc
    Meta,

    // REPL or shell prompts or similar
    Prompt,
}

/// Run a syntax highlighter over the provided source code
pub fn run(language: &str, source: &str) -> Result<impl Display, Box<dyn Error>> {
    let desc = SourceDescription::new(language);
    let mut source: Vec<_> = source.lines().collect();

    let mut hide_lines = vec![];
    if let Some(hide_prefix) = desc.properties.get("hide") {
        for (idx, line) in source.iter_mut().enumerate() {
            if let Some(stripped) = line.strip_prefix(hide_prefix) {
                hide_lines.push(idx);
                *line = stripped;
            }
        }
    }

    let source = source.join("\n");
    let mut source = match desc.language.as_str() {
        // "rs" | "rust" => todo!(),
        // "lua" => todo!(),
        _ => unknown::highlight(&source),
    };

    let highlights: Vec<usize> = desc
        .properties
        .get("highlight")
        .copied()
        .unwrap_or_default()
        .split(",")
        .flat_map(|n| n.parse())
        .collect();
    for (idx, line) in source.lines.iter_mut().enumerate() {
        if highlights.contains(&(idx + 1)) {
            line.kind = HighlightedLineKind::Highlighted;
        }
    }

    let mut idx = 0;
    source.lines.retain(|_| {
        let ret = hide_lines.contains(&idx);
        idx += 1;
        !ret
    });

    Ok(source)
}

impl<'a> SourceDescription<'a> {
    fn new(language: &'a str) -> Self {
        let parts: Vec<_> = language.split(' ').collect();

        let mut properties = HashMap::new();
        for &part in &parts[1..] {
            if let Some((l, r)) = part.split_once('=') {
                properties.insert(l, r);
            } else {
                properties.insert(part, "");
            }
        }

        Self {
            language: parts[0].to_string(),
            properties,
        }
    }
}

impl Display for HighlightedSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<pre class='highlighted'><code><ol>")?;

        for (idx, line) in self.lines.iter().enumerate() {
            if idx != 0 {
                writeln!(f)?;
            }
            line.fmt(f)?;
        }

        write!(f, "</ol></code></pre>")
    }
}

impl Display for HighlightedLine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<li")?;

        match self.kind {
            HighlightedLineKind::None => write!(f, ">")?,
            HighlightedLineKind::Highlighted => write!(f, " class='line-highlight'>")?,
            HighlightedLineKind::DiffAddition => write!(f, " class='line-add'>")?,
            HighlightedLineKind::DiffRemoval => write!(f, " class='line-remove'>")?,
        }

        // ensure there is something to make the line heights in css work right
        if self.content.is_empty() {
            write!(f, " ")?;
        }

        for (scope, text) in &self.content {
            if *scope == HighlightScope::None {
                escape_html_body_text(FmtWriter(&mut *f), text)?;
            } else {
                let class = format!("{:?}", scope).to_lowercase();
                write!(f, "<span class='{class}'>")?;
                escape_html_body_text(FmtWriter(&mut *f), text)?;
                write!(f, "</span>")?;
            }
        }

        write!(f, "</li>")
    }
}
