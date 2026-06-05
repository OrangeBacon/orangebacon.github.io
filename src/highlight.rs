use std::{collections::HashMap, error::Error, fmt::Display, sync::LazyLock};

use pulldown_cmark_escape::{FmtWriter, escape_html_body_text};
use regex::Regex;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent};

/// A single tree sitter language
struct Language {
    cfg: HighlightConfiguration,
    names: Vec<&'static str>,
}

/// Syntax Highlighter
pub struct SyntaxHighlighter {
    highlighter: tree_sitter_highlight::Highlighter,

    languages: HashMap<&'static str, Language>,
}

/// A highlighted source file
struct Highlight<'a> {
    source: &'a str,
    iter: Vec<HighlightEvent>,
    names: &'a [&'static str],
}

impl SyntaxHighlighter {
    /// Create a highlighter with all supported languages
    pub fn new() -> Result<Self, Box<dyn Error>> {
        Ok(Self {
            highlighter: tree_sitter_highlight::Highlighter::new(),
            languages: HashMap::from([
                ("rs", Language::rust()?),
                ("sh", Language::bash()?),
                ("c", Language::c()?),
                ("c++", Language::cpp()?),
                ("js", Language::js()?),
                ("ts", Language::ts()?),
                ("zig", Language::zig()?),
                ("c#", Language::c_sharp()?),
            ]),
        })
    }

    /// Highlight a source file
    pub fn highlight<'a>(
        &'a mut self,
        lang: &str,
        source: &'a str,
    ) -> Result<impl Display + 'a, Box<dyn Error>> {
        let Some(lang) = self.languages.get(lang) else {
            eprintln!("Unrecognised language: {lang}");
            return Ok(Highlight {
                source,
                iter: vec![HighlightEvent::Source {
                    start: 0,
                    end: source.len(),
                }],
                names: &[],
            });
        };
        let iter = self
            .highlighter
            .highlight(&lang.cfg, source.trim().as_bytes(), None, |l| {
                self.languages.get(l).map(|l| &l.cfg)
            })?
            .flatten()
            .collect();

        Ok(Highlight {
            source,
            iter,
            names: &lang.names,
        })
    }
}

impl Display for Highlight<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<pre><code><span class='line'>")?;
        let mut current = None;
        for &ev in &self.iter {
            match ev {
                HighlightEvent::Source { start, end } => {
                    let s = &self.source[start..end];
                    let lines = s.lines().collect::<Vec<_>>();
                    let len = lines.len();
                    for (idx, line) in lines.into_iter().enumerate() {
                        escape_html_body_text(FmtWriter(&mut *f), line)?;
                        if idx >= len - 1 && !s.ends_with('\n') {
                            break;
                        }
                        if current.is_some() {
                            write!(f, "</span>")?;
                        }
                        write!(f, "</span>\n<span class='line'>")?;
                        if let Some(curr) = current {
                            let name: &'static str = self.names[curr];
                            write!(f, "<span class='")?;
                            write!(f, "{}", &name.split('.').collect::<Vec<_>>().join(" "))?;
                            write!(f, "'>")?;
                        }
                    }
                }
                HighlightEvent::HighlightStart(highlight) => {
                    write!(f, "<span class='")?;
                    write!(
                        f,
                        "{}",
                        &self.names[highlight.0]
                            .split('.')
                            .collect::<Vec<_>>()
                            .join(" "),
                    )?;
                    write!(f, "'>")?;
                    current = Some(highlight.0);
                }
                HighlightEvent::HighlightEnd => {
                    write!(f, "</span>")?;
                    current = None;
                }
            }
        }

        write!(f, "</code></pre>")
    }
}

impl Language {
    /// Get a highlighter for the rust language
    fn rust() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_rust::LANGUAGE.into(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            "",
        )?;

        let names = get_names(tree_sitter_rust::HIGHLIGHTS_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for shell scripts
    fn bash() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_bash::LANGUAGE.into(),
            "bash",
            tree_sitter_bash::HIGHLIGHT_QUERY,
            "",
            "",
        )?;

        let names = get_names(tree_sitter_bash::HIGHLIGHT_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for c
    fn c() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_c::LANGUAGE.into(),
            "c",
            tree_sitter_c::HIGHLIGHT_QUERY,
            "",
            "",
        )?;

        let names = get_names(tree_sitter_c::HIGHLIGHT_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for c++
    fn cpp() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_cpp::LANGUAGE.into(),
            "c++",
            tree_sitter_cpp::HIGHLIGHT_QUERY,
            "",
            "",
        )?;

        let names = get_names(tree_sitter_cpp::HIGHLIGHT_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for js
    fn js() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_javascript::LANGUAGE.into(),
            "js",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::INJECTIONS_QUERY,
            tree_sitter_javascript::LOCALS_QUERY,
        )?;

        let names = get_names(tree_sitter_javascript::HIGHLIGHT_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for ts
    fn ts() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "ts",
            tree_sitter_typescript::HIGHLIGHTS_QUERY,
            "",
            tree_sitter_typescript::LOCALS_QUERY,
        )?;

        let names = get_names(tree_sitter_typescript::HIGHLIGHTS_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for zig
    fn zig() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_zig::LANGUAGE.into(),
            "zig",
            tree_sitter_zig::HIGHLIGHTS_QUERY,
            tree_sitter_zig::INJECTIONS_QUERY,
            "",
        )?;

        let names = get_names(tree_sitter_zig::HIGHLIGHTS_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }

    /// Get a highlighter for c#
    fn c_sharp() -> Result<Self, Box<dyn Error>> {
        let mut cfg = HighlightConfiguration::new(
            tree_sitter_c_sharp::LANGUAGE.into(),
            "c#",
            tree_sitter_c_sharp::HIGHLIGHTS_QUERY,
            "",
            "",
        )?;

        let names = get_names(tree_sitter_c_sharp::HIGHLIGHTS_QUERY);

        cfg.configure(&names);

        Ok(Language { cfg, names })
    }
}

/// Get the syntax highlighting scopes from a language's highlights query
fn get_names(query: &str) -> Vec<&str> {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"@[\w\.]+").unwrap());

    RE.find_iter(query).map(|m| &m.as_str()[1..]).collect()
}
