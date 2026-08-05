use crate::highlight::{HighlightScope, HighlightedLine, HighlightedLineKind, HighlightedSource};

pub fn highlight(source: &str) -> HighlightedSource {
    HighlightedSource {
        lines: source
            .lines()
            .map(|l| HighlightedLine {
                kind: HighlightedLineKind::None,
                content: vec![(HighlightScope::None, l.to_string())],
            })
            .collect(),
    }
}
