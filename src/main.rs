extern crate clap;
extern crate shapefile;

use clap::{App, Arg};
use std::path::Path;

fn main() {
    let matches = App::new("shpinfo")
        .version("0.1.0")
        .author("Shiwei Wang <wsw0108@gmail.com>")
        .about("Display info about shapefile")
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("file to parse"),
        )
        .get_matches();
    let file = matches.value_of("FILE").unwrap();
    let path = Path::new(file);
    let reader = shapefile::Reader::from_path(&path).unwrap();
    let mut count = 0;
    for _ in reader {
        count += 1;
    }
    println!("{}", count);
}
