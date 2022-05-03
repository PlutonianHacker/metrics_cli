mod args;

use std::{
    collections::BTreeMap,
    fs::{self},
    io::{self, Write},
    path::PathBuf,
};

use args::Args;
use regex::Regex;

#[macro_use]
extern crate lazy_static;

lazy_static! {
    static ref NEWLINES: Regex = Regex::new("(:?.)+[\n]").unwrap();
    static ref SEMI_COLONS: Regex = Regex::new(";").unwrap();
    static ref FIXMES: Regex = Regex::new("// FIXME:").unwrap();
    static ref TODOS: Regex = Regex::new("// TODO:").unwrap();
}

#[derive(Debug)]
pub struct Metrics {
    pub newlines: usize,
    pub semicolons: usize,
    pub num_files: usize,
    pub todos: usize,
    pub fixmes: usize,
    pub files: Cache,
}

#[derive(Debug)]
pub struct Cache {
    pub total_lines: usize,
    pub total_size: u64,
    pub lines: BTreeMap<usize, String>,
    pub sizes: BTreeMap<u64, String>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            lines: BTreeMap::new(),
            sizes: BTreeMap::new(),
            total_lines: 0,
            total_size: 0,
        }
    }

    pub fn insert_line(&mut self, k: usize, v: String) {
        self.lines.insert(k, v);
    }

    pub fn insert_size(&mut self, k: u64, v: String) {
        self.sizes.insert(k, v);
    }

    /// Get the largest file by lines of code.
    pub fn max_lines(&self) -> (usize, &str) {
        let pair = self.lines.iter().next_back().unwrap();
        (*pair.0, &pair.1)
    }

    /// Get the file with the fewest lines of code.
    pub fn min_lines(&self) -> (usize, &str) {
        let pair = self.lines.iter().next().unwrap();
        (*pair.0, &pair.1)
    }

    /// Find the average number of lines per file.
    pub fn average_lines(&self) -> usize {
        let mut sum = 0;

        self.lines.iter().for_each(|(x, _)| {
            sum += x;
        });

        sum / self.lines.len()
    }

    /// Get the largest file by lines of bytes.
    pub fn max_size(&self) -> (u64, &str) {
        let pair = self.sizes.iter().next_back().unwrap();
        (*pair.0, &pair.1)
    }

    /// Get the file with the fewest lines of code.
    pub fn min_size(&self) -> (u64, &str) {
        let pair = self.sizes.iter().next().unwrap();
        (*pair.0, &pair.1)
    }

    /// Find the average number of lines per file.
    pub fn average_size(&self) -> u64 {
        let mut sum = 0_usize;

        self.sizes.iter().for_each(|(x, _)| {
            sum += *x as usize;
        });

        (sum / self.lines.len()) as u64
    }
}

/// A file's metadata.
pub struct Meta {
    /// The path to the file.
    path: String,
    /// The size of the file, in bytes.
    size: u64,
}

impl Meta {
    pub fn new(path: String, size: u64) -> Self {
        Meta { path, size }
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{:>8.2} B", bytes)
    } else if bytes < 1024_u64.pow(2) {
        format!("{:>8.2} KiB", bytes as f32 / 1024 as f32)
    } else if bytes < 1024_u64.pow(3) {
        format!(
            "{:>8.2} MiB",
            bytes as f32 / 1024_f32.powf(2.) as f32
        )
    } else if bytes < 1024_u64.pow(4) {
        format!(
            "{:>8.2} GiB",
            bytes as f32 / 1024_f32.powf(3.) as f32
        )
    } else if bytes < 1024_u64.pow(5) {
        format!(
            "{:>8.2} TiB",
            bytes as f32 / 1024_f32.powf(4.) as f32
        )
    } else if bytes < 1024_u64.pow(6) {
        format!(
            "{:>8.2} PiB",
            bytes as f32 / 1024_f32.powf(5.) as f32
        )
    } else {
        format!("{bytes}")
    }
}
pub struct File(String, Meta);

fn read_dirs<Path: Into<PathBuf>>(
    paths: Vec<Path>,
    cache: &mut Vec<File>,
    extensions: &[String],
) -> io::Result<()> {
    for path in paths {
        read_dir_recursive(path, cache, extensions)?;
    }

    Ok(())
}

fn read_dir_recursive<Path: Into<PathBuf>>(
    path: Path,
    cache: &mut Vec<File>,
    extensions: &[String],
) -> io::Result<()> {
    let path = path.into();

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                read_dir_recursive(entry.path(), cache, extensions)?;
            } else {
                if entry.path().extension().is_none() {
                    continue;
                }

                if extensions.contains(
                    &entry
                        .path()
                        .extension()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .to_string(),
                ) {
                    let file = fs::read_to_string(entry.path())?;
                    let size = fs::metadata(entry.path())?.len();
                    cache.push(File(
                        file,
                        Meta::new(entry.path().to_str().unwrap().into(), size),
                    ));
                }
            }
        }
    }

    Ok(())
}

fn metrics(files: Vec<File>) -> Metrics {
    let mut newlines = 0;
    let mut semicolons = 0;
    let mut todos = 0;
    let mut fixmes = 0;
    let mut cache = Cache::new();
    let num_files = files.len();

    for file in files {
        let (file, meta) = (file.0, file.1);

        let re: Vec<_> = NEWLINES.captures_iter(&file).collect();
        newlines += re.len();

        let re: Vec<_> = SEMI_COLONS.captures_iter(&file).collect();
        semicolons += re.len();

        let re: Vec<_> = TODOS.captures_iter(&file).collect();
        todos += re.len();

        let re: Vec<_> = FIXMES.captures_iter(&file).collect();
        fixmes += re.len();

        let len = file.split("\n").collect::<Vec<&str>>().len();

        cache.total_lines += len;
        cache.total_size += meta.size;

        cache.insert_line(len, meta.path.clone());
        cache.insert_size(meta.size, meta.path);
    }

    Metrics {
        newlines,
        semicolons,
        num_files,
        todos,
        fixmes,
        files: cache,
    }
}

fn main() -> io::Result<()> {
    let args = Args::new();
    let mut files = Vec::<File>::new();

    read_dirs(args.paths, &mut files, &args.extensions)?;

    let metrics = metrics(files);

    let io = io::stdout();
    let mut handle = io.lock();

    writeln!(handle, "semicolons{:>10}", metrics.semicolons)?;
    writeln!(handle, "newlines{:>12}", metrics.newlines)?;
    writeln!(handle, "todos{:>15}", metrics.todos)?;
    writeln!(handle, "fixmes{:>14}", metrics.fixmes)?;
    writeln!(
        handle,
        "files{:>15} files{}",
        metrics.num_files,
        format_bytes(metrics.files.total_size)
    )?;

    let smallest = metrics.files.min_lines();
    let largest = metrics.files.max_lines();

    let padding = 5;

    writeln!(handle, "\nlines{:>15}", metrics.files.total_lines)?;
    writeln!(
        handle,
        "smallest file{:>7} lines{}{}{}",
        smallest.0,
        format_bytes(metrics.files.min_size().0),
        " ".repeat(padding),
        smallest.1
    )?;
    writeln!(
        handle,
        "largest file{:>8} lines{}{}{}",
        largest.0,
        format_bytes(metrics.files.max_size().0),
        " ".repeat(padding),
        largest.1
    )?;
    writeln!(
        handle,
        "average{:>13} lines{}",
        metrics.files.average_lines(),
        format_bytes(metrics.files.average_size()),
    )?;
    
    Ok(())
}
