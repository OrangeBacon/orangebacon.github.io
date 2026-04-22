//! This static site builder aims to be an opinionated site builder, purely
//! for this site.  I am trying to keep this minimal, so I focus less on this
//! pile of code, and more on the content on the site.  (lets see how this goes)

mod file;
mod markdown;
mod output_template;
mod plain_text;
mod template;

use std::{
    error::Error,
    fs::{self, DirEntry},
    path::Component,
};

use crate::{
    file::SiteEntries, markdown::MarkdownHandler, output_template::OutputTemplate,
    plain_text::TextHandler, template::TemplateHandler,
};

pub const OUTPUT_DIR: &str = "docs";

fn main() -> Result<(), Box<dyn Error>> {
    remove_old()?;

    // create the new output directory, if already exists ignore the error
    _ = fs::create_dir(OUTPUT_DIR);

    let mut entries = SiteEntries::new();
    entries.handler(TemplateHandler);
    entries.handler(OutputTemplate);
    entries.handler(MarkdownHandler::new());
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
        ".vscode",       // visual studio code metadata
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

/// remove the output directory to clear the previous build files, however keep
/// the `.git` file from git worktree
fn remove_old() -> Result<(), Box<dyn Error>> {
    let Ok(dir) = fs::read_dir(OUTPUT_DIR) else {
        // the output dir doesn't exist, no issue
        return Ok(());
    };

    for item in dir {
        let item = item?;
        let path = item.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else if !path.ends_with(".git") {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}
