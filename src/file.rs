use std::{
    error::Error,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use crate::OUTPUT_DIR;

/// Handle all file processing
pub fn process_file(path: &Path) -> Result<(), Box<dyn Error>> {
    // for now just copy input to output without processing
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

    buf_write.write_all(input.as_bytes())?;

    Ok(())
}

/// Get the path of a file, following processing
/// - If path starts with `./root` return `./`
/// - Prepend `OUTPUT_DIR`
fn output_path(mut path: &Path) -> PathBuf {
    if let Ok(strip) = path.strip_prefix("./root") {
        path = strip;
    }

    PathBuf::from(OUTPUT_DIR).join(path)
}
