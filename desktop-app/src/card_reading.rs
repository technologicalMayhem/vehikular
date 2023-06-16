use crate::parsing::combine_registrations;
use iso7816_tlv::{
    ber::{Tag, Tlv, Value},
    TlvError,
};
use log::debug;
use pcsc::{Card, MAX_BUFFER_SIZE};
use shared::data::Registration;
use std::collections::HashMap;
use thiserror::Error;

use crate::card_reading::select_file::{retrieve_file, File};

/// Read all regisration files from the card and combines their data into the [``Registration``] struct for easier use.
///
/// # Errors
///
/// This function will return an error if an error occured whilst reading the card.
pub fn read_card(card: &Card) -> Result<Registration, CardReadingError> {
    if !is_evrc_card(card)? {
        Err(CardReadingError::NotAneVrc)?;
    }

    let files = read_files(
        card,
        vec![
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

/// The errors that can occur during the card reading process.
#[derive(Debug, Error)]
pub enum CardReadingError {
    /// The card is not a eVRC card.
    #[error("The card is not a eVRC card.")]
    NotAneVrc,
    /// Got an unsuccessful response from card. Contains the command sent and the response received.
    #[error("Got an unsuccessful response from card. Command: {0:?}. Response: {1:?}")]
    UnsuccesfulResponseFromCard(Vec<u8>, Vec<u8>),
    /// Failed to read a TLV. This is a wrapped [``iso7816_tlv::Error::TlvError``] from the [``iso7816_tlv``] crate.
    #[error("Could not read TLV")]
    TlvReadError(#[from] TlvError),
    /// A error occured whilst reading from the card. This is a wrapped [``pcsc::Error``] from the [``pcsc``] crate.
    #[error("A error occured whilst reading the card: {0}")]
    CardReadingError(#[from] pcsc::Error),
    /// Failed to read data from the FCP template. This is a wrapped [``FcpParseError``].
    #[error("Could not parse the FCP template for {0:?}")]
    FailedToReadFcp(File, FcpParseError),
}

/// Tests whether or not this a eVRC card.
///
/// # Errors
///
/// This function will return an error if an error occured whilst reading the card.
fn is_evrc_card(card: &Card) -> Result<bool, CardReadingError> {
    const SELECT_EVRC_APPLICATION: &[u8; 17] = &[
        0x00, 0xA4, 0x04, 0x00, 0x0B, 0xA0, 0x00, 0x00, 0x04, 0x56, 0x45, 0x56, 0x52, 0x2D, 0x30,
        0x31, 0x00,
    ];
    const SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE: &[u8; 15] = &[
        0x6F, 0x0D, 0x84, 0x0B, 0xA0, 0x00, 0x00, 0x04, 0x56, 0x45, 0x56, 0x52, 0x2D, 0x30, 0x31,
    ];

    let response = run_apdu(card, &SELECT_EVRC_APPLICATION.to_vec())?;
    Ok(response == SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE)
}

/// .
///
/// # Errors
///
/// This function will return an error if an error occured whilst reading the card.
fn read_files(card: &Card, files: Vec<File>) -> Result<HashMap<File, Vec<u8>>, CardReadingError> {
    let mut map = HashMap::new();

    for file in files {
        let bytes = retrieve_file(card, file)?;
        map.insert(file, bytes);
    }

    Ok(map)
}

/// Sends a application protocol data unit (APDU) to the smartcard and returns it's response.
///
/// # Errors
///
/// This function will return an error if an error occured whilst reading the card.
fn run_apdu(card: &Card, apdu: &Vec<u8>) -> Result<Vec<u8>, CardReadingError> {
    debug!("Sending APDU: {apdu:?}");

    let mut response_buf = [0; MAX_BUFFER_SIZE];
    let response = card.transmit(apdu, &mut response_buf)?;

    debug!("Got response: {response:?}");

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

/// Checks if a response from the eVRC card has [0x90, 0x00], which indicates successful processing, at the end.
fn is_sucessful(response: &[u8]) -> bool {
    let len = response.len();
    if len <= 1 {
        return false;
    }

    response[len - 2..len] == [0x90, 0x00]
}

/// Contains functions and data related to selecting and reading files from the eVRC smartcard.
mod select_file {
    use color_eyre::Result;
    use iso7816_tlv::ber::Tlv;
    use pcsc::Card;

    use crate::card_reading::{run_apdu, CardReadingError, FcpTemplate};

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

    /// Represetns the different files that cam be read from the card.
    #[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
    pub enum File {
        /// Contains crytographic information that can be used to confirm the cards authenticity.
        FSOd,
        /// Contains the bulk of the vehicle information stored on the chip.
        RegistrationA,
        /// Contaisn some additional vehicles information, this seems to be a EU standard.
        RegistrationB,
        /// Contains data extensions which might contain additional information dependant on the vehicle.
        RegistrationC,
    }

    impl File {
        /// Returns a reference to the binary identifier of this [`File`].
        #[must_use]
        pub fn binary_identifier(&self) -> &[u8; 2] {
            match self {
                File::FSOd => &[0x00, 0x1D],
                File::RegistrationA => &[0xD0, 0x01],
                File::RegistrationB => &[0xD0, 0x11],
                File::RegistrationC => &[0xD0, 0x21],
            }
        }

        /// Returns the corresponding file for the given identifier. Panics if the given byte sequence does not coreespond to a file type.
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

    /// Retrieves a file from the smartcard.
    ///
    /// # Errors
    ///
    /// This function will return an error if an error occured whilst reading the card.
    pub fn retrieve_file(card: &Card, file: File) -> Result<Vec<u8>, CardReadingError> {
        let fcp = select_file(card, file)?;
        read_file(card, &fcp)
    }

    fn select_file(card: &Card, file: File) -> Result<FcpTemplate, CardReadingError> {
        let file_id = file.binary_identifier();
        let apdu = vec![CLA, INS, P1, P2, LC, file_id[0], file_id[1], LE];
        let response = run_apdu(card, &apdu)?;
        let (fcp, _) = Tlv::parse(&response);

        FcpTemplate::try_from(fcp?).map_err(|err| CardReadingError::FailedToReadFcp(file, err))
    }

    /// Reads out the contents of the file stored on the smart card for the given [``FcpTemplate``].
    ///
    /// # Errors
    ///
    /// This function will return an error if an error occured whilst reading the card.
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

/// Describes a file on the eVRC smartcard.
struct FcpTemplate {
    file_size: FileSize,
}

impl FcpTemplate {
    /// The identifier byte of the FCP template.
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
