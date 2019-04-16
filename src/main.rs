use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;

use clap::{App, Arg};

mod dbf;

const HEADER_SIZE: i32 = 100;
const INDEX_RECORD_SIZE: usize = 2 * std::mem::size_of::<i32>();

fn main() -> Result<(), Error> {
    let matches = App::new("shpinfo")
        .version("0.1.3")
        .about("Display info about shapefile")
        .arg(
            Arg::with_name("FILE")
                .required(true)
                .takes_value(true)
                .index(1)
                .help("file/directory to parse"),
        )
        .get_matches();

    let file = matches.value_of("FILE").unwrap();
    let path = Path::new(file);
    if path.is_dir() {
        let mut total_count = 0;
        for entry in path.read_dir()? {
            if let Ok(entry) = entry {
                let is_file = entry.file_type()?.is_file();
                let name = entry.file_name().to_str().unwrap().to_string();
                let is_shp = name.ends_with(".shp");
                if is_file && is_shp {
                    let count = process_file(&entry.path())?;
                    total_count += count;
                }
            }
        }
        println!("Total Features: {}", total_count);
    } else {
        process_file(path)?;
    }

    Ok(())
}

#[derive(Debug)]
enum Error {
    ShapefileError(shapefile::Error),
    DbfError(dbf::Error),
    IoErr(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoErr(e)
    }
}

impl From<shapefile::Error> for Error {
    fn from(e: shapefile::Error) -> Self {
        Error::ShapefileError(e)
    }
}

impl From<dbf::Error> for Error {
    fn from(e: dbf::Error) -> Self {
        Error::DbfError(e)
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

fn process_file(file: &Path) -> Result<i32, Error> {
    println!("File: {:?}", file);
    let shape_path = file;
    let shx_path = shape_path.with_extension("shx");
    let feature_count = read_feature_count(&shx_path)?;
    let dbf_path = shape_path.with_extension("dbf");
    let fields_info = read_dbf_fields(&dbf_path)?;
    let reader = shapefile::Reader::from_path(&shape_path)?;
    let header = reader.header();
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
        let f = File::open(cpg_path)?;
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        println!("Code Page: {}", line.trim());
    }
    let prj_path = shape_path.with_extension("prj");
    if prj_path.exists() {
        let f = File::open(prj_path)?;
        let mut reader = BufReader::new(f);
        let mut line = String::new();
        reader.read_line(&mut line)?;
        println!("Projection: {}", line.trim());
    }
    println!();

    Ok(feature_count)
}