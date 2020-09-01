extern crate nom;
use nom::{bytes::streaming::take, character::complete::multispace0};

extern crate byteorder;
use byteorder::{BigEndian, ByteOrder};

mod card_value;
mod error;
mod primary_header;

use primary_header::PrimaryHeader;
#[derive(Debug)]
pub struct Fits<'a> {
    pub header: PrimaryHeader<'a>,
    pub data: DataType<'a>,
}

trait DataUnit<'a>: std::marker::Sized {
    type Item: Default;

    fn parse(buf: &'a [u8], num_items: usize) -> Result<Self, Error<'a>> {
        let num_bytes_per_item = std::mem::size_of::<Self::Item>();
        let num_bytes = num_items * num_bytes_per_item;
        let (_, raw_bytes) = take(num_bytes)(buf)?;

        let data = Self::new(raw_bytes, num_items);
        Ok(data)
    }

    fn new(raw_bytes: &'a [u8], num_items: usize) -> Self;
}

#[derive(Debug)]
pub struct DataUnitU8<'a>(pub &'a [u8]);
impl<'a> DataUnit<'a> for DataUnitU8<'a> {
    type Item = u8;
    fn new(raw_bytes: &'a [u8], _num_items: usize) -> Self {
        DataUnitU8(raw_bytes)
    }
}

#[derive(Debug)]
pub struct DataUnitI16(pub Vec<i16>);
impl<'a> DataUnit<'a> for DataUnitI16 {
    type Item = i16;
    fn new(raw_bytes: &[u8], num_items: usize) -> Self {
        let mut dst: Vec<Self::Item> = vec![Self::Item::default(); num_items];
        BigEndian::read_i16_into(raw_bytes, &mut dst);

        DataUnitI16(dst)
    }
}

#[derive(Debug)]
pub struct DataUnitI32(pub Vec<i32>);
impl<'a> DataUnit<'a> for DataUnitI32 {
    type Item = i32;
    fn new(raw_bytes: &[u8], num_items: usize) -> Self {
        let mut dst: Vec<Self::Item> = vec![Self::Item::default(); num_items];
        BigEndian::read_i32_into(raw_bytes, &mut dst);

        DataUnitI32(dst)
    }
}

#[derive(Debug)]
pub struct DataUnitI64(pub Vec<i64>);
impl<'a> DataUnit<'a> for DataUnitI64 {
    type Item = i64;
    fn new(raw_bytes: &[u8], num_items: usize) -> Self {
        let mut dst: Vec<Self::Item> = vec![Self::Item::default(); num_items];
        BigEndian::read_i64_into(raw_bytes, &mut dst);

        DataUnitI64(dst)
    }
}
#[derive(Debug)]
pub struct DataUnitF32(pub Vec<f32>);
impl<'a> DataUnit<'a> for DataUnitF32 {
    type Item = f32;
    fn new(raw_bytes: &[u8], num_items: usize) -> Self {
        let mut dst: Vec<Self::Item> = vec![Self::Item::default(); num_items];
        BigEndian::read_f32_into(raw_bytes, &mut dst);

        DataUnitF32(dst)
    }
}
#[derive(Debug)]
pub struct DataUnitF64(pub Vec<f64>);
impl<'a> DataUnit<'a> for DataUnitF64 {
    type Item = f64;
    fn new(raw_bytes: &[u8], num_items: usize) -> Self {
        let mut dst: Vec<Self::Item> = vec![Self::Item::default(); num_items];
        BigEndian::read_f64_into(raw_bytes, &mut dst);

        DataUnitF64(dst)
    }
}

use error::Error;
use primary_header::BitpixValue;
impl<'a> Fits<'a> {
    pub fn from_bytes_slice(buf: &'a [u8]) -> Result<Fits<'a>, Error<'a>> {
        let (buf, header) = PrimaryHeader::new(&buf)?;

        // At this point the header is valid
        let num_items = (0..header.get_naxis())
            .map(|idx| header.get_axis_size(idx).unwrap())
            .fold(1, |mut total, val| {
                total *= val;
                total
            });

        multispace0(buf)?;

        // Read the byte data stream in BigEndian order conformly to the spec
        let data = match header.get_bitpix() {
            BitpixValue::U8 => DataType::U8(DataUnitU8::parse(buf, num_items)?),
            BitpixValue::I16 => DataType::I16(DataUnitI16::parse(buf, num_items)?),
            BitpixValue::I32 => DataType::I32(DataUnitI32::parse(buf, num_items)?),
            BitpixValue::I64 => DataType::I64(DataUnitI64::parse(buf, num_items)?),
            BitpixValue::F32 => DataType::F32(DataUnitF32::parse(buf, num_items)?),
            BitpixValue::F64 => DataType::F64(DataUnitF64::parse(buf, num_items)?),
        };

        Ok(Fits { header, data })
    }
}

#[derive(Debug)]
pub enum DataType<'a> {
    U8(DataUnitU8<'a>),
    I16(DataUnitI16),
    I32(DataUnitI32),
    I64(DataUnitI64),
    F32(DataUnitF32),
    F64(DataUnitF64),
}

#[cfg(test)]
mod tests {
    use super::primary_header::{BitpixValue, FITSHeaderKeyword};
    use super::{Fits, PrimaryHeader};
    use std::io::Read;
    #[test]
    fn test_fits_tile() {
        use std::fs::File;
        let f = File::open("misc/Npix208.fits").unwrap();
        let bytes: Result<Vec<_>, _> = f.bytes().collect();
        let buf = bytes.unwrap();
        let Fits { header, .. } = Fits::from_bytes_slice(&buf).unwrap();
        let PrimaryHeader { cards, .. } = header;

        let cards_expect = vec![
            ("SIMPLE", FITSHeaderKeyword::Simple),
            ("BITPIX", FITSHeaderKeyword::Bitpix(BitpixValue::F32)),
            ("NAXIS", FITSHeaderKeyword::Naxis(2)),
            (
                "NAXIS1",
                FITSHeaderKeyword::NaxisSize {
                    name: "NAXIS1",
                    idx: 1,
                    size: 64,
                },
            ),
            (
                "NAXIS2",
                FITSHeaderKeyword::NaxisSize {
                    name: "NAXIS2",
                    idx: 2,
                    size: 64,
                },
            ),
        ];
        assert_eq!(cards, cards_expect);
        println!("{:?}", cards);
    }

    #[test]
    fn test_fits_tile2() {
        use std::fs::File;
        use crate::DataType;
        let  f  = File::open("misc/Npix282.fits").unwrap();
        let  bytes: Result<Vec<_>, _> = f.bytes().collect();
        let  buf = bytes.unwrap();
        let  Fits { data, .. } = Fits::from_bytes_slice(&buf).unwrap();
        
        match data {
            DataType::F32(v) => {
                //println!("{:?}", v);
            },
            _ => unreachable!()
        };
        
    }

    #[test]
    fn test_fits_tile3() {
        use std::fs::File;
        use crate::DataType;
        let  f  = File::open("misc/Npix4906.fits").unwrap();
        let  bytes: Result<Vec<_>, _> =  f.bytes().collect();
        let  buf  =  bytes.unwrap();
        let  Fits { data, .. } =  Fits::from_bytes_slice(&buf).unwrap();
        
        match data {
            DataType::I16(v) => {
                println!("{:?}", v);
            },
            _ => unreachable!()
        };
    }

    /*#[test]
    fn test_fits_tile4() {
        use std::fs::File;
        use crate::DataType;
        let  f  = File::open("misc/Npix8.fits").unwrap();
        let  bytes: Result<Vec<_>, _> =  f.bytes().collect();
        let  buf  =  bytes.unwrap();
        let  Fits { data, .. } =  Fits::from_bytes_slice(&buf).unwrap();
        
        match data {
            DataType::I16(v) => {
                println!("{:?}", v);
            },
            _ => unreachable!()
        };
        
    }*/
}
