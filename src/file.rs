use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html::push_html};

use crate::OUTPUT_DIR;

/// Handle all file processing
pub fn process_file(path: &Path) -> Result<(), Box<dyn Error>> {
    let output_path = output_path(path);

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let output_file = fs::OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(output_path)?;
    let mut buf_write = BufWriter::new(output_file);

    let input = fs::read_to_string(path)?;

    if path.extension().map(|e| e == "md").unwrap_or(false) {
        process_markdown(&input, buf_write)?;
    } else {
        buf_write.write_all(input.as_bytes())?;
    }

    Ok(())
}

/// Get the path of a file, following processing
/// - If path starts with `./root` return `./`
/// - If the file extension is ".md", replace it with ".html"
/// - Prepend `OUTPUT_DIR`
fn output_path(path: &Path) -> PathBuf {
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
fn process_markdown(input: &str, mut writer: impl std::io::Write) -> Result<(), Box<dyn Error>> {
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
    let template = std::fs::read_to_string(metadata["template"])?;

    let mut output = template;
    for (key, value) in metadata {
        output = output.replace(&format!("{{% {key} %}}"), value);
    }

    writer.write_all(output.as_bytes())?;

    Ok(())
}
