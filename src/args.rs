use std::path::PathBuf;

use clap::{Arg, Command};

pub struct Args {
    pub extensions: Vec<String>,
    pub paths: Vec<PathBuf>,
}

impl Args {
    pub fn new() -> Self {
        let app = Command::new("metrics")
            .about("A command line utility for reporting metrics.")
            .arg(
                Arg::new("extensions")
                    .help("list of file extensions to report")
                    .required(false)
                    .default_missing_value(""),
            )
            .arg(
                Arg::new("path")
                    .help("The directories to report")
                    .required(true)
                    .min_values(1),
            );

        let matches = app.get_matches();

        let paths: Vec<PathBuf> = matches
            .values_of("path")
            .unwrap()
            .into_iter()
            .map(PathBuf::from)
            .collect();

        let extensions: Vec<String> = matches
            .values_of("extensions")
            .unwrap()
            .map(String::from)
            .collect();

        Args { extensions, paths }
    }
}
