use thiserror::Error;

use crate::select_file::File;

#[derive(Debug, Error)]
pub enum ParseTlvError {
    #[error("The input data is of unexpected type. Expected: {0:X}. Got: {1:X}")]
    WrongType(u8, u8),
    #[error("The input data ended unexpectedly")]
    UnexpectedEof,
}

#[derive(Debug)]
pub struct Tlv<'a> {
    pub t: &'a u8,
    pub l: &'a u8,
    pub v: &'a [u8],
}

impl<'a> Tlv<'a> {
    pub fn parse(data: &'a [u8]) -> Result<(Self, usize), ParseTlvError> {
        if data.len() < 2 {
            return Err(ParseTlvError::UnexpectedEof);
        }
        let t = &data[0];
        let l = &data[1];
        if data.len() < 2 + *l as usize {
            return Err(ParseTlvError::UnexpectedEof);
        }

        let v = &data[2..2 + *l as usize];

        Ok((Tlv { t, l, v }, 2 + *l as usize))
    }

    pub fn parse_all(data: &'a [u8]) -> Result<Vec<Self>, ParseTlvError> {
        let mut tlvs: Vec<Tlv<'a>> = Vec::new();
        let mut next = 0;

        while next < data.len() {
            println!("Next: {next}");
            let (tlv, n) = Self::parse(&data[next..data.len()])?;
            println!("{tlv:#?}");
            println!("Add: {n}");
            tlvs.push(tlv);
            next += n;
        }

        Ok(tlvs)
    }
}

#[derive(Debug)]
pub struct FcpTemplate {
    pub file_id: File,
    pub file_size: u16,
}

impl FcpTemplate {
    pub fn parse(data: &[u8]) -> Result<Self, ParseTlvError> {
        let (fcp, remainder) = Tlv::parse(data)?;
        if remainder > 0 {
            eprintln!("Data remaining after parsing FCP template. This is not good. Remaining {remainder}");
        }

        if *fcp.t != 0x62 {
            return Err(ParseTlvError::WrongType(0x62, *fcp.t));
        }

        let (file_id, next) = Tlv::parse(fcp.v)?;
        let (file_size, _) = Tlv::parse(&fcp.v[next..fcp.v.len()])?;

        let file_id = File::from_identifier([file_id.v[0], file_id.v[1]]);
        let file_size = u16::from_be_bytes([file_size.v[0], file_size.v[1]]);

        Ok(FcpTemplate { file_id, file_size })
    }
}
