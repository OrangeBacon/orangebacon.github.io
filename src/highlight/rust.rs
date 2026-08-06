use std::ops::Range;

use proc_macro2::{TokenStream, TokenTree};
use quote::ToTokens;

use crate::highlight::{HighlightScope, HighlightedLine, HighlightedLineKind, HighlightedSource};

pub fn highlight(source: &str) -> HighlightedSource {
    let file = match syn::parse_file(source) {
        Ok(f) => f,
        Err(e) => panic!("Rust parse error: {e}\nSource:\n{source}"),
    };

    let stream = file.to_token_stream();

    let iter = TokenIter::new(source, stream);

    let mut lines = vec![];
    let mut line = vec![];
    let mut current_line = 1;
    for tok in iter.output.into_iter() {
        if tok.line != current_line {
            current_line = tok.line;
            lines.push(HighlightedLine {
                content: std::mem::take(&mut line),
                kind: HighlightedLineKind::None,
            });
        }

        line.push((tok.kind, tok.text));
    }

    if !line.is_empty() {
        lines.push(HighlightedLine {
            content: std::mem::take(&mut line),
            kind: HighlightedLineKind::None,
        });
    }

    HighlightedSource { lines }
}

#[derive(Debug)]
struct HighlightToken {
    kind: HighlightScope,
    text: String,
    line: usize,
}

#[derive(Debug, Default)]
struct TokenIter<'a> {
    output: Vec<HighlightToken>,
    current_byte: usize,
    current_line_no: usize,
    source: &'a str,
}

impl<'a> TokenIter<'a> {
    /// Create and run a recursive iterator over the token stream and its associated
    /// source code.
    fn new(source: &'a str, stream: TokenStream) -> Self {
        let mut this = Self {
            output: vec![],
            current_byte: 0,
            current_line_no: 1,
            source,
        };
        this.run(stream);
        this
    }

    /// Recurse over the given token stream's tokens.
    fn run(&mut self, stream: TokenStream) {
        for tok in stream {
            match tok {
                TokenTree::Group(group) => self.group(group),
                TokenTree::Ident(ident) => self.ident(ident),
                TokenTree::Punct(punct) => self.punct(punct),
                TokenTree::Literal(literal) => self.literal(literal),
            }
        }
    }

    /// Process a group token
    fn group(&mut self, group: proc_macro2::Group) {
        self.add_token(
            group.span_open().start().line,
            group.span_open().byte_range(),
            HighlightScope::Punctuation,
        );
        self.run(group.stream());
        self.add_token(
            group.span_close().start().line,
            group.span_close().byte_range(),
            HighlightScope::Punctuation,
        );
    }

    /// Process an ident token
    fn ident(&mut self, ident: proc_macro2::Ident) {
        let span = ident.span();
        let s = &self.source[span.byte_range()];
        let kind = match () {
            _ if KEYWORDS.contains(&s) => HighlightScope::Keyword,
            _ if TYPES.contains(&s) => HighlightScope::Type,
            _ if LITERAL.contains(&s) => HighlightScope::Literal,
            _ if s
                .chars()
                .all(|c| c == '_' || c.is_numeric() || c.is_uppercase()) =>
            {
                HighlightScope::Constant
            }
            _ if s
                .chars()
                .find(|&c| !(c == '_' || c.is_numeric()))
                .is_some_and(|c| c.is_uppercase()) =>
            {
                HighlightScope::Class
            }
            _ => HighlightScope::Variable,
        };
        self.add_token(span.start().line, span.byte_range(), kind);
    }

    /// Process a punct token
    fn punct(&mut self, punct: proc_macro2::Punct) {
        let span = punct.span();
        let kind = match "!#$%&'*+-./:<=>?@^|~".contains(punct.as_char()) {
            true => HighlightScope::Operator,
            false => HighlightScope::Punctuation,
        };
        self.add_token(span.start().line, span.byte_range(), kind);
    }

    /// Process a literal token
    fn literal(&mut self, literal: proc_macro2::Literal) {
        let span = literal.span();
        let s = &self.source[span.byte_range()];
        let kind = match s.contains(['"', '\'']) {
            true => HighlightScope::String,
            false => HighlightScope::Number,
        };
        self.add_token(span.start().line, span.byte_range(), kind);
    }

    /// Adds a token to the output.
    ///
    /// If the token is not continuous with the previous one, adds comment tokens
    /// between this token and the previous one, so that all characters from the
    /// source code are included.
    ///
    /// If the token covers multiple lines, it is split into single line tokens,
    /// with the `\n` character missing.
    fn add_token(&mut self, line_no: usize, range: Range<usize>, kind: HighlightScope) {
        let new_start = range.start;

        let comment_range = self.current_byte..new_start;
        if !comment_range.is_empty() {
            // recurse to handle multi-line comments.  the condition to recurse is
            // by-definition false on the recursive call, max-depth = 2.
            self.add_token(self.current_line_no, comment_range, HighlightScope::Comment);
        }

        let text = &self.source[range.start..range.end];
        for (idx, line) in text.lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            self.push_token(HighlightToken {
                kind,
                text: line.to_string(),
                line: line_no + idx,
            });
        }

        let line = self.output.last().map(|l| l.line).unwrap_or(0);
        self.current_line_no = line;
        self.current_byte = range.end;
    }

    /// Check if a token can be combined with one that is already in the output
    /// stream to make a more specific token kind.
    ///
    /// Always results in pushing all the involved tokens to the output stream
    fn push_token(&mut self, tok: HighlightToken) {
        let Some(last) = self.output.last_mut() else {
            self.output.push(tok);
            return;
        };

        if tok.line != last.line {
            self.output.push(tok);
        } else if last.text == "'" && is_ident(&tok) {
            last.kind = HighlightScope::Symbol;
            last.text.push_str(&tok.text);
        } else if tok.text == "!" && is_ident(last) {
            last.kind = HighlightScope::Macro;
            last.text.push_str(&tok.text);
        } else {
            self.output.push(tok);
        }
    }
}

/// Is this token an identifier, for the purposes of combining it to make a macro
/// call or a lifetime symbol
fn is_ident(tok: &HighlightToken) -> bool {
    matches!(
        tok.kind,
        HighlightScope::Class
            | HighlightScope::Constant
            | HighlightScope::Keyword
            | HighlightScope::Type
            | HighlightScope::Variable
            | HighlightScope::Literal
    )
}

const KEYWORDS: &[&str] = &[
    "_",
    "as",
    "async",
    "await",
    "break",
    "const",
    "continue",
    "crate",
    "dyn",
    "else",
    "enum",
    "extern",
    "false",
    "fn",
    "for",
    "if",
    "impl",
    "in",
    "let",
    "loop",
    "match",
    "mod",
    "move",
    "mut",
    "pub",
    "ref",
    "return",
    "static",
    "struct",
    "super",
    "trait",
    "true",
    "type",
    "unsafe",
    "use",
    "where",
    "while",
    "abstract",
    "become",
    "box",
    "do",
    "final",
    "gen",
    "macro",
    "override",
    "priv",
    "try",
    "typeof",
    "unsized",
    "virtual",
    "yield",
    "macro_rules",
    "raw",
    "safe",
    "union",
];

const TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize", "f16",
    "f32", "f64", "f128", "bool", "char", "str", "Self",
];

const LITERAL: &[&str] = &["true", "false", "None", "Some", "Err", "Ok", "self"];
