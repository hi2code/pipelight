use project_root::get_project_root;
use rev_buf_reader::RevBufReader;
use std::error::Error;
use std::fs::File;
use std::io::BufRead;
use std::path::Path;

pub mod git;
pub mod logger;

/// Return project root path as string
pub fn get_root() -> Result<String, Box<dyn Error>> {
    let root = get_project_root()?;
    let to_str_result = root.to_str();
    match to_str_result {
        Some(res) => return Ok(res.to_owned()),
        None => {
            let message = "Internal error: Couldn't find project root";
            // warn!("{}", message);
            return Err(Box::from(message));
        }
    };
}

pub fn read_last_line(path: &Path) -> Result<String, Box<dyn Error>> {
    let file = File::open(path)?;
    let buf = RevBufReader::new(file);
    let mut lines = buf.lines();
    let last_line = lines.next().unwrap().unwrap();
    Ok(last_line)
}