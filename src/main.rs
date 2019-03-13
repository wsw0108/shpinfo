extern crate clap;
extern crate shapefile;
extern crate dbase;

use clap::{App, Arg};
use std::io::prelude::*;
use std::path::Path;
use std::io::BufReader;
use std::fs::File;
use std::collections::HashMap;

const HEADER_SIZE: i32 = 100;
const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

fn main() {
    let matches = App::new("shpinfo")
        .version("0.1.0")
        .author("Shiwei Wang <wsw0108@gmail.com>")
        .about("Display info about shapefile")
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("show verbose output")
        )
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("file to parse"),
        )
        .get_matches();
    let verbose = matches.is_present("verbose");
    let file = matches.value_of("FILE").unwrap();
    let shape_path = Path::new(file);
    let mut feature_count: i32 = 0;
    let mut fields: HashMap<String, String> = HashMap::new();
    let shx_path = shape_path.with_extension("shx");
    if shx_path.exists() {
        let mut source = BufReader::new(File::open(&shx_path).unwrap());
        let header = shapefile::header::Header::read_from(&mut source).unwrap();
        let num_shapes = ((header.file_length * 2) - HEADER_SIZE) / INDEX_RECORD_SIZE as i32;
        feature_count = num_shapes;
    }
    let dbf_path = shape_path.with_extension("dbf");
    if dbf_path.exists() {
        let mut _source = dbase::Reader::from_path(dbf_path);
        // TODO: read field info
    }
    if !shx_path.exists() || verbose {
        let mut count = 0;
        let reader = shapefile::Reader::from_path(&shape_path).unwrap();
        for result in reader.iter_shapes_and_records().unwrap() {
            match result {
                Err(_) => {}
                Ok((_shape, record)) => {
                    // println!("{}", _shape);
                    // for (name, value) in record {
                    //     println!("{} -> {:?}", name, value);
                    // }
                    for (name, _value) in record {
                        fields.insert(name, "".to_string());
                    }
                }
            }
            count += 1;
        }
        if !shx_path.exists() {
            feature_count = count;
        }
    }
    println!("Feature Count: {}", feature_count);
    if verbose {
        println!("Fields:");
        for (field, _) in fields {
            println!("  {}", field);
        }
    }
    let cpg_path = shape_path.with_extension("cpg");
    if cpg_path.exists() {
        let f = File::open(cpg_path).unwrap();
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        let _len = reader.read_line(&mut line).unwrap();
        println!("Code Page: {}", line.trim());
    }
    let prj_path = shape_path.with_extension("prj");
    if prj_path.exists() {
        let f = File::open(prj_path).unwrap();
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        let _len = reader.read_line(&mut line).unwrap();
        println!("Projection: {}", line.trim());
    }
}
