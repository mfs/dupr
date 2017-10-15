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
        .arg(Arg::with_name("noempty").short("n").long("noempty").help(
            "Exclude zero length files",
        ))
        .arg(Arg::with_name("summary").short("s").long("summary").help(
            "Print out summary information after duplicates",
        ))
        .get_matches()
}


fn collect_paths(path: &str, noempty: bool, stats: &mut Stats) -> HashMap<u64, Vec<PathBuf>> {

    let mut length_paths = HashMap::new();
    let spinner = ["|", "/", "-", "\\"];

    for (idx, entry) in WalkDir::new(path).into_iter().enumerate() {
        eprint!("\rBuilding file list {}", spinner[idx % spinner.len()]);
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                eprintln!("dupr: {}", err);
                continue;
            }
        };

        if !entry.file_type().is_file() {
            continue;
        }

        let metadata = match entry.metadata() {
            Ok(e) => e,
            Err(err) => {
                eprintln!("dupr: {}", err);
                continue;
            }
        };

        if noempty && metadata.len() == 0 {
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
    eprint!("\r");
    length_paths
}


fn main() {
    let matches = parse_args();

    let mut stats: Stats = Default::default();

    let files = collect_paths(
        matches.value_of("DIR").unwrap(),
        matches.is_present("noempty"),
        &mut stats,
    );

    let mut len_hash_path = HashMap::new();

    // iterate over length buckets when no. files > 1
    for (len, paths) in files.iter().filter(|&(_, v)| v.len() > 1) {

        let mut hardlinks = HashSet::new();

        for path in paths {
            // only save if not a hardlink to existing file
            // we are still sometimes hashing a file here
            // that we don't need to when all the paths
            // are hardlinks to the same file.
            let metadata = match path.metadata() {
                Ok(m) => m,
                Err(err) => {
                    eprintln!("dupr: {}", err);
                    continue;
                }
            };

            let dev_inode = (metadata.st_dev(), metadata.st_ino());

            if hardlinks.contains(&dev_inode) {
                continue;
            }

            hardlinks.insert(dev_inode);

            len_hash_path
                .entry((len, hash_file(&path)))
                .or_insert(Vec::new())
                .push(path);
        }
    }

    for paths in len_hash_path.values().filter(|p| p.len() > 1) {
        for path in paths {
            println!("{}", path.display());
        }
        println!();
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
