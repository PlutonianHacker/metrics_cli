mod args;

use std::{
    fs,
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
}

fn read_dirs<Path: Into<PathBuf>>(
    paths: Vec<Path>,
    cache: &mut Vec<String>,
    extensions: &[String],
) -> io::Result<()> {
    for path in paths {
        read_dir_recursive(path, cache, extensions)?;
    }

    Ok(())
}

fn read_dir_recursive<Path: Into<PathBuf>>(
    path: Path,
    cache: &mut Vec<String>,
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
                    cache.push(file);
                }
            }
        }
    }

    Ok(())
}

fn metrics(files: Vec<String>) -> Metrics {
    let mut newlines = 0;
    let mut semicolons = 0;
    let mut todos = 0;
    let mut fixmes = 0;
    let num_files = files.len();

    for file in files {
        let re: Vec<_> = NEWLINES.captures_iter(&file).collect();
        newlines += re.len();

        let re: Vec<_> = SEMI_COLONS.captures_iter(&file).collect();
        semicolons += re.len();

        let re: Vec<_> = TODOS.captures_iter(&file).collect();
        todos += re.len();

        let re: Vec<_> = FIXMES.captures_iter(&file).collect();
        fixmes += re.len();
    }

    Metrics {
        newlines,
        semicolons,
        num_files,
        todos,
        fixmes,
    }
}

fn main() -> io::Result<()> {
    let args = Args::new();

    let mut files = Vec::<String>::new();
    //let extensions = &["rs"];

    read_dirs(args.paths, &mut files, &args.extensions)?;

    let metrics = metrics(files);

    let io = io::stdout();
    let mut handle = io.lock();

    writeln!(handle, "semicolons{:>10}", metrics.semicolons)?;
    writeln!(handle, "newlines{:>12}", metrics.newlines)?;
    writeln!(handle, "todos{:>15}", metrics.todos)?;
    writeln!(handle, "fixmes{:>14}", metrics.fixmes)?;
    writeln!(handle, "files{:>15}", metrics.num_files)?;

    Ok(())
}
