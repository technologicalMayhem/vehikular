#![feature(iter_intersperse)]

use iso7816_tlv::{
    ber::{Tag, Tlv, Value},
    TlvError,
};
use parsing::combine_registrations;
use pcsc::{Card, MAX_BUFFER_SIZE};
use shared::data::Registration;
use std::{collections::HashMap, sync::Mutex};
use thiserror::Error;

use crate::select_file::{retrieve_file, File};

mod parsing;

pub static DEBUG: Mutex<bool> = Mutex::new(false);

pub fn read_card(card: &Card) -> Result<Registration, CardReadingError> {
    if !is_evrc_card(card)? {
        Err(CardReadingError::NotAneVrc)?;
    }

    let files = read_files(
        card,
        vec![
            File::FSOd,
            File::RegistrationA,
            File::RegistrationB,
            File::RegistrationC,
        ],
    )?;

    let registrations: Vec<Tlv> = files
        .iter()
        .filter_map(|(file, bytes)| {
            if *file == File::FSOd {
                None
            } else {
                Some(Tlv::parse_all(bytes))
            }
        })
        .collect::<Vec<Vec<Tlv>>>()
        .concat();

    Ok(combine_registrations(&registrations))
}

#[derive(Debug, Error)]
pub enum CardReadingError {
    #[error("The card is not a eVRC card.")]
    NotAneVrc,
    #[error("Got unsuccessful response from card. Command: {0:?}. Response: {1:?}")]
    UnsuccesfulResponseFromCard(Vec<u8>, Vec<u8>),
    #[error("Could not read TLV")]
    TlvReadError(#[from] TlvError),
    #[error("Could not parse the FCP template for {0:?}")]
    FailedToReadFcp(File, FcpParseError),
}

fn is_evrc_card(card: &Card) -> Result<bool, CardReadingError> {
    let response = run_apdu(card, &SELECT_EVRC_APPLICATION.to_vec())?;
    Ok(response == SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE)
}

fn read_files(card: &Card, files: Vec<File>) -> Result<HashMap<File, Vec<u8>>, CardReadingError> {
    let mut map = HashMap::new();

    for file in files {
        let bytes = retrieve_file(card, file)?;
        map.insert(file, bytes);
    }

    Ok(map)
}

fn run_apdu(card: &Card, apdu: &Vec<u8>) -> Result<Vec<u8>, CardReadingError> {
    if is_debug() {
        println!("Sending APDU: {apdu:?}");
    }

    let mut response_buf = [0; MAX_BUFFER_SIZE];
    let response = match card.transmit(apdu, &mut response_buf) {
        Ok(response) => response,
        Err(err) => {
            eprintln!("Failed to transmit APDU command to card: {err}");
            std::process::exit(1);
        }
    };

    if is_sucessful(response) {
        // We lop off the response bytes if it's succesful as they are not needed.
        Ok(response[0..response.len() - 2].to_vec())
    } else {
        Err(CardReadingError::UnsuccesfulResponseFromCard(
            apdu.clone(),
            response.to_vec(),
        ))
    }
}

fn is_debug() -> bool {
    *DEBUG.lock().expect("Could not read debug value")
}

fn is_sucessful(response: &[u8]) -> bool {
    let len = response.len();
    if len <= 1 {
        return false;
    }

    response[len - 2..len] == [0x90, 0x00]
}

pub const SELECT_EVRC_APPLICATION: &[u8; 17] = &[
    0x00, 0xA4, 0x04, 0x00, 0x0B, 0xA0, 0x00, 0x00, 0x04, 0x56, 0x45, 0x56, 0x52, 0x2D, 0x30, 0x31,
    0x00,
];
pub const SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE: &[u8; 15] = &[
    0x6F, 0x0D, 0x84, 0x0B, 0xA0, 0x00, 0x00, 0x04, 0x56, 0x45, 0x56, 0x52, 0x2D, 0x30, 0x31,
];

pub mod select_file {
    use color_eyre::Result;
    use iso7816_tlv::ber::Tlv;
    use pcsc::Card;

    use crate::{run_apdu, CardReadingError, FcpTemplate};

    /// Class byte. 00 indicates no secure messaging (SM).
    const CLA: u8 = 0x00;
    /// Instruction byte. A4 means 'Select File'
    const INS: u8 = 0xA4;
    /// Parameter 1 byte. Set to 02, unsure why.
    const P1: u8 = 0x02;
    /// Parameter 2 byte. Set to 04, unsure why.
    const P2: u8 = 0x04;
    /// Content length byte. Indicates the length of the file name. Should always be 2 for our purposes.
    const LC: u8 = 0x02;
    /// Expected length byte. How long we expect the response to be. 00 indicates that we expect any length.
    const LE: u8 = 0x00;

    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub enum File {
        FSOd,
        RegistrationA,
        RegistrationB,
        RegistrationC,
    }

    impl File {
        #[must_use]
        pub fn get_indetifier(&self) -> &[u8; 2] {
            match self {
                File::FSOd => &[0x00, 0x1D],
                File::RegistrationA => &[0xD0, 0x01],
                File::RegistrationB => &[0xD0, 0x11],
                File::RegistrationC => &[0xD0, 0x21],
            }
        }

        #[must_use]
        pub fn from_identifier(data: [u8; 2]) -> Self {
            match data {
                [0x00, 0x1D] => File::FSOd,
                [0xD0, 0x01] => File::RegistrationA,
                [0xD0, 0x11] => File::RegistrationB,
                [0xD0, 0x21] => File::RegistrationC,
                _ => unimplemented!(),
            }
        }
    }

    pub fn retrieve_file(card: &Card, file: File) -> Result<Vec<u8>, CardReadingError> {
        let fcp = select_file(card, file)?;
        read_file(card, &fcp)
    }

    fn select_file(card: &Card, file: File) -> Result<FcpTemplate, CardReadingError> {
        let file_id = file.get_indetifier();
        let apdu = vec![CLA, INS, P1, P2, LC, file_id[0], file_id[1], LE];
        let response = run_apdu(card, &apdu)?;
        let (fcp, _) = Tlv::parse(&response);

        FcpTemplate::try_from(fcp?)
            .map_err(|err| CardReadingError::FailedToReadFcp(file, err))
    }

    #[allow(clippy::cast_possible_truncation)]
    fn read_file(card: &Card, fcp: &FcpTemplate) -> Result<Vec<u8>, CardReadingError> {
        let mut data: Vec<u8> = Vec::new();

        for offset in (0..fcp.file_size.0).step_by(256) {
            let offset: [u8; 2] = offset.to_be_bytes();
            let apdu = [0x00, 0xB0, offset[0], offset[1], 0x00].to_vec();
            let response = &mut run_apdu(card, &apdu)?;
            data.append(response);
        }

        Ok(data)
    }
}

struct FcpTemplate {
    file_size: FileSize,
}

impl FcpTemplate {
    fn tag() -> i32 {
        0x62
    }

    fn get_inner(inner: &[Tlv], value: i32) -> Result<&Tlv, <FcpTemplate as TryFrom<Tlv>>::Error> {
        let tag = Tag::try_from(value)?;
        let file_size_tlv = inner
            .iter()
            .find(|tlv| *tlv.tag() == tag)
            .ok_or(FcpParseError::MissingTlvInValue)?;
        Ok(file_size_tlv)
    }
}

impl TryFrom<Tlv> for FcpTemplate {
    type Error = FcpParseError;

    fn try_from(value: Tlv) -> std::result::Result<Self, Self::Error> {
        if *value.tag() != Tag::try_from(Self::tag())? {
            Err(FcpParseError::WrongTag(Self::tag(), value.tag().clone()))?;
        }

        let Value::Constructed(inner) = value.value() else {
            Err(FcpParseError::NoInnerContructed)?
        };

        let file_size_tlv = Self::get_inner(inner, FileSize::tag())?;

        let file_size = FileSize::try_from(file_size_tlv)?;

        Ok(FcpTemplate { file_size })
    }
}

struct FileId(File);

impl FileId {
    fn tag() -> i32 {
        0x83
    }
}

impl TryFrom<&Tlv> for FileId {
    type Error = FcpParseError;

    fn try_from(value: &Tlv) -> std::result::Result<Self, Self::Error> {
        if *value.tag() != Tag::try_from(Self::tag())? {
            Err(FcpParseError::WrongTag(Self::tag(), value.tag().clone()))?;
        }
        let Value::Primitive(data) = value.value() else {
            Err(FcpParseError::NoInnerPrimitive)?
        };

        if data.len() != 2 {
            Err(FcpParseError::InvalidValue("FileId".into(), value.length()))?;
        }

        let mut id: [u8; 2] = [0; 2];
        id.copy_from_slice(data);

        Ok(FileId(File::from_identifier(id)))
    }
}

struct FileSize(u16);

impl FileSize {
    fn tag() -> i32 {
        0x80
    }
}

impl TryFrom<&Tlv> for FileSize {
    type Error = FcpParseError;

    fn try_from(value: &Tlv) -> std::result::Result<Self, Self::Error> {
        if *value.tag() != Tag::try_from(Self::tag())? {
            Err(FcpParseError::WrongTag(Self::tag(), value.tag().clone()))?;
        }
        let Value::Primitive(data) = value.value() else {
            Err(FcpParseError::NoInnerPrimitive)?
        };

        if data.len() != 2 {
            Err(FcpParseError::InvalidValue(
                "FileSize".into(),
                value.length(),
            ))?;
        }

        let mut size: [u8; 2] = [0; 2];
        size.copy_from_slice(data);

        Ok(FileSize(u16::from_be_bytes(size)))
    }
}

#[derive(Debug, Error)]
pub enum FcpParseError {
    #[error("An error occured when parsing a tlv.")]
    TlvError(#[from] iso7816_tlv::TlvError),
    #[error("A unexpected tag was encountered. Expected {0}. Got {1}.")]
    WrongTag(i32, Tag),
    #[error("Missing tlv inside another tlv's value.")]
    MissingTlvInValue,
    #[error("Expected constructed data inside tlv but found none")]
    NoInnerContructed,
    #[error("Expected primitive data inside tlv but found none")]
    NoInnerPrimitive,
    #[error("The value is invalid and cannot be parsed. Length of {0}'s value field: {1}.")]
    InvalidValue(String, usize),
}
