extern crate clap;
extern crate walkdir;
extern crate twox_hash;

use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::os::linux::fs::MetadataExt;

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


fn collect_paths(path: &str, stats: &mut Stats) -> HashMap<u64, Vec<PathBuf>> {

    let mut length_paths = HashMap::new();

    for entry in WalkDir::new(path) {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                println!("dupr: {}", err); // stderr
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(e) => e,
            Err(err) => {
                println!("dupr: {}", err); //std err
                continue;
            }
        };

        if metadata.len() == 0 {
            continue;
        }

        stats.file_count += 1;
        stats.total_size += metadata.len();

        let path = entry.path().to_path_buf();

        length_paths
            .entry(metadata.len())
            .or_insert(Vec::new())
            .push(path);
    }

    length_paths
}


fn main() {
    let matches = parse_args();

    let mut stats: Stats = Default::default();

    let files = collect_paths(matches.value_of("DIR").unwrap(), &mut stats);

    for f_paths in files.values().filter(|x| x.len() > 1) {

        // Avoid hardlinks to same inode
        let mut hardlinks = Vec::new();
        let mut di = HashSet::new();
        for f_path in f_paths {
            let metadata = match f_path.metadata() {
                Ok(m) => m,
                Err(err) => {
                    println!("dupr: {}", err);
                    continue;
                }
            };
            let dev_inode = (metadata.st_dev(), metadata.st_ino());
            if !di.contains(&dev_inode) {
                hardlinks.push(f_path);
                di.insert(dev_inode);
            }
        }

        let mut hashes = HashMap::new();
        for f_path in hardlinks {
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
