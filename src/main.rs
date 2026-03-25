//! This static site builder aims to be an opinionated site builder, purely
//! for this site.  I am trying to keep this minimal, so I focus less on this
//! pile of code, and more on the content on the site.  (lets see how this goes)

mod file;
mod markdown;
mod plain_text;

use std::{
    error::Error,
    fs::{self, DirEntry},
    path::Component,
};

use crate::{file::SiteEntries, markdown::MarkdownHandler, plain_text::TextHandler};

pub const OUTPUT_DIR: &str = "docs";

fn main() -> Result<(), Box<dyn Error>> {
    // remove the old output directory, ignore if it fails
    _ = fs::remove_dir_all(OUTPUT_DIR);

    // create the new output directory
    fs::create_dir(OUTPUT_DIR)?;

    let mut entries = SiteEntries::new();
    entries.handler(MarkdownHandler);
    entries.handler(TextHandler);

    for entry in fs::read_dir(".")?.flatten().filter(file_filter) {
        process_entry(&mut entries, &entry)?;
    }

    entries.process()?;

    Ok(())
}

/// Get all files that should be considered when generating the site.  Includes
/// files that might not be present in the output, e.g. template files.
fn file_filter(entry: &DirEntry) -> bool {
    let path = entry.path();

    // block top level files
    if path.is_file() {
        return false;
    }

    // filter by the directory name
    let components: Vec<_> = path.components().collect();
    let Some(Component::Normal(filter_dir)) = components.get(1) else {
        return true;
    };
    let name = filter_dir.to_string_lossy();
    ![
        OUTPUT_DIR,      // the output directory
        ".git",          // git metadata
        ".devcontainer", // used for github codespaces
        "target",        // rust build files
        "src",           // source code to the generator
    ]
    .contains(&name.as_ref())
}

/// Process a directory entry
fn process_entry(entries: &mut SiteEntries, entry: &DirEntry) -> Result<(), Box<dyn Error>> {
    let path = entry.path();

    if path.is_dir() {
        for dir in fs::read_dir(path)?.flatten() {
            process_entry(entries, &dir)?;
        }
    } else {
        let content = fs::read_to_string(&path)?;
        entries.add(path, content);
    }

    Ok(())
}
