extern crate clap;
extern crate walkdir;
extern crate twox_hash;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use clap::{App, Arg};
use walkdir::WalkDir;
use twox_hash::XxHash;
use std::hash::Hasher;

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

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = entry.metadata().unwrap();

        let path = entry.path().to_path_buf();

        files.entry(metadata.len()).or_insert(Vec::new()).push(path);
    }

    for (f_len, f_paths) in files {
        println!("=== {} - {} ===", f_len, f_paths.len());
        for f_path in f_paths {
            println!("    {} - {:x}", f_path.display(), hash_file(&f_path));
        }
    }
}

fn hash_file<P: AsRef<Path>>(path: P) -> u64 {
    let mut file = File::open(path).unwrap();

    let mut hash = XxHash::with_seed(0);

    let mut buffer = [0; 32 * 1024];

    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 {
            break;
        }
        hash.write(&buffer[0..n]);
    }

    hash.finish()
}
