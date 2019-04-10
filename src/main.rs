extern crate clap;
extern crate dbase;
extern crate shapefile;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg};

mod dbf;

const HEADER_SIZE: i32 = 100;
const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

fn main() {
    let matches = App::new("shpinfo")
        .version("0.1.2")
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
    let shx_path = shape_path.with_extension("shx");
    let feature_count = read_feature_count(&shx_path).unwrap();
    let dbf_path = shape_path.with_extension("dbf");
    let fields_info = read_dbf_fields(&dbf_path).unwrap();
    let reader = shapefile::Reader::from_path(&shape_path).unwrap();
    let header = reader.header().clone();
    if verbose {
        for result in reader.iter_shapes_and_records().unwrap() {
            match result {
                Err(_) => {}
                Ok((_shape, _record)) => {}
            }
        }
    }
    println!("Shape Type: {}", header.shape_type);
    println!("Extent: [{}, {}, {}, {}]",
             header.point_min[0], header.point_min[1],
             header.point_max[0], header.point_max[1]);
    println!("Feature Count: {}", feature_count);
    println!("Fields:");
    for field in fields_info {
        println!("  {}: {:?}", field.name, field.field_type);
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

fn read_feature_count(shx_path: &Path) -> Result<i32, shapefile::Error> {
    let mut source = BufReader::new(File::open(shx_path)?);

    let header = shapefile::header::Header::read_from(&mut source)?;

    Ok(((header.file_length * 2) - HEADER_SIZE) / INDEX_RECORD_SIZE as i32)
}

fn read_dbf_fields(dbf_path: &Path) -> Result<Vec<dbf::RecordFieldInfo>, dbf::Error> {
    let mut fields_info = vec![];

    let mut source = BufReader::new(File::open(dbf_path)?);

    let header = dbf::Header::read_from(&mut source)?;
    let num_fields = (header.offset_to_first_record as usize - dbf::Header::SIZE) / dbf::RecordFieldInfo::SIZE;

    for _ in 0..num_fields {
        let info = dbf::RecordFieldInfo::read_from(&mut source)?;
        fields_info.push(info);
    };

    Ok(fields_info)
}