extern crate clap;
extern crate walkdir;

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


    for entry in WalkDir::new(matches.value_of("DIR").unwrap()) {
        let entry = entry.unwrap();
        let metadata = entry.metadata().unwrap();
        println!("{} - {}", entry.path().display(), metadata.len());
    }
}
