extern crate clap;
extern crate walkdir;
extern crate twox_hash;

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::{App, Arg, ArgMatches};
use walkdir::WalkDir;
use twox_hash::XxHash;
use std::hash::Hasher;

#[derive(Default)]
struct Stats {
    file_count: u64,
    total_size: u64,
    duplicate_count: u64,
}

fn parse_args<'a>() -> ArgMatches<'a> {
    App::new("dupr")
        .version("0.1.0")
        .author("Mike Sampson")
        .about("Duplicate file finder")
        .arg(
            Arg::with_name("DIR")
                .help("Directory to process")
                .required(true),
        )
        .arg(Arg::with_name("summary").short("s").long("summary").help(
            "Print out summary information after duplicates",
        ))
        .get_matches()
}


fn collect_files(path: &str, stats: &mut Stats) -> HashMap<u64, Vec<PathBuf>> {

    let mut files = HashMap::new();

    for entry in WalkDir::new(path) {
        let entry = entry.unwrap();

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = entry.metadata().unwrap();

        if metadata.len() == 0 {
            continue;
        }

        stats.file_count += 1;
        stats.total_size += metadata.len();

        let path = entry.path().to_path_buf();

        files.entry(metadata.len()).or_insert(Vec::new()).push(path);
    }

    files
}


fn main() {
    let matches = parse_args();

    let mut stats: Stats = Default::default();

    let files = collect_files(matches.value_of("DIR").unwrap(), &mut stats);

    for f_paths in files.values().filter(|x| x.len() > 1) {

        let mut hashes = HashMap::new();
        for f_path in f_paths {
            hashes
                .entry(hash_file(&f_path))
                .or_insert(Vec::new())
                .push(f_path);
        }

        for (hash, paths) in hashes.iter().filter(|&(_, v)| v.len() > 1) {
            stats.duplicate_count += paths.len() as u64;
            for p in paths {
                println!("{:016x} - {}", hash, p.display());
            }
        }
    }

    if matches.is_present("summary") {
        println!(
            "Processed {} files with a total size of {} bytes. {} duplicates found.",
            stats.file_count,
            stats.total_size,
            stats.duplicate_count
        );
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
