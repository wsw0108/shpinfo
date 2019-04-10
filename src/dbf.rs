// from https://github.com/tmontaigu/dbase-rs

use std::io::Read;
use byteorder::{LittleEndian, ReadBytesExt};

/// Errors that may happen when reading a .dbf
#[derive(Debug)]
pub enum Error {
    /// Wrapper of `std::io::Error` to forward any reading/writing error
    IoError (std::io::Error),
    /// Wrapper to forward errors whe trying to parse a float from the file
    ParseFloatError(std::num::ParseFloatError),
    /// Wrapper to forward errors whe trying to parse an integer value from the file
    ParseIntError(std::num::ParseIntError),
    /// The Field as an invalid FieldType
    InvalidFieldType(char),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(e)
    }
}

impl From<std::num::ParseFloatError> for Error {
    fn from(p: std::num::ParseFloatError) -> Self {
        Error::ParseFloatError(p)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(p: std::num::ParseIntError) -> Self {
        Error::ParseIntError(p)
    }
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum FieldType {
    Character,
    Currency,
    Numeric,
    Float,
    Date,
    DateTime,
    Double,
    Integer,
    Logical,
    Memo,
    General,
    BinaryCharacter,
    BinaryMemo,
    Picture,
    Varbinary,
    BinaryVarchar,
}

impl FieldType {
    pub fn from(c: char) -> Option<FieldType> {
        match c {
            'C' => Some(FieldType::Character),
            'Y' => Some(FieldType::Currency),
            'N' => Some(FieldType::Numeric),
            'F' => Some(FieldType::Float),
            'D' => Some(FieldType::Date),
            'T' => Some(FieldType::DateTime),
            'B' => Some(FieldType::Double),
            'I' => Some(FieldType::Integer),
            'L' => Some(FieldType::Logical),
            'M' => Some(FieldType::Memo),
            'G' => Some(FieldType::General),
            //'C' => Some(FieldType::BinaryCharacter), ??
            //'M' => Some(FieldType::BinaryMemo),
            _ => None,
        }
    }

    pub fn try_from(c: char) -> Result<FieldType, Error> {
        match Self::from(c) {
            Some(t) => Ok(t),
            None => Err(Error::InvalidFieldType(c))
        }
    }
}

pub(crate) struct Header {
    #[allow(dead_code)]
    pub(crate) num_records: u32,
    pub(crate) offset_to_first_record: u16,
    #[allow(dead_code)]
    pub(crate) size_of_record: u16,
}

impl Header {
    pub(crate) const SIZE: usize = 32;

    pub(crate) fn read_from<T: Read>(source: &mut T) -> Result<Self, std::io::Error> {
        let mut skip = [0u8; 4];
        source.read_exact(&mut skip)?; //level + last date

        let num_records = source.read_u32::<LittleEndian>()?;
        let offset_to_first_record = source.read_u16::<LittleEndian>()?;
        let size_of_record = source.read_u16::<LittleEndian>()?;

        let mut skip = [0u8; 20];
        source.read_exact(&mut skip)?; //level + last date

        Ok(Self {
            num_records,
            offset_to_first_record,
            size_of_record,
        })
    }
}

pub struct RecordFieldInfo {
    /// The name of the field
    pub name: String,
    /// The field type
    pub field_type: FieldType,
    pub record_length: u8,
    pub num_decimal_places: u8,
}

impl RecordFieldInfo {
    pub(crate) const SIZE: usize = 32;

    pub(crate) fn read_from<T: Read>(source: &mut T) -> Result<Self, Error> {
        let mut name = [0u8; 11];
        source.read_exact(&mut name)?;
        let field_type = source.read_u8()?;

        let mut displacement_field = [0u8; 4];
        source.read_exact(&mut displacement_field)?;

        let record_length = source.read_u8()?;
        let num_decimal_places = source.read_u8()?;

        let mut skip = [0u8; 14];
        source.read_exact(&mut skip)?;

        let s = String::from_utf8_lossy(&name).trim_matches(|c| c == '\u{0}').to_owned();
        let field_type = FieldType::try_from(field_type as char)?;
        Ok(Self{
            name: s,
            field_type,
            record_length,
            num_decimal_places
        })
    }
}
