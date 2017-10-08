extern crate clap;
extern crate walkdir;

use std::collections::HashMap;

use clap::{App, Arg};
use walkdir::WalkDir;

fn main() {
    let matches = App::new("dupr")
        .version("0.1.0")
        .author("Mike Sampson")
        .about("Duplicate file finder")
        .arg(
            Arg::with_name("DIR")
                .help("Directory to process")
                .required(true),
        )
        .get_matches();

    let mut files = HashMap::new();

    for entry in WalkDir::new(matches.value_of("DIR").unwrap()) {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();

        let path = entry.path().to_path_buf();

        files.entry(metadata.len()).or_insert(Vec::new()).push(path);
    }

    for (f_len, f_paths) in files {
        println!("=== {} - {} ===", f_len, f_paths.len());
        for f_path in f_paths {
            println!("    {}", f_path.display());
        }
    }
}
