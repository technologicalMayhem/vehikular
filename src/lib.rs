#![feature(iter_intersperse)]

use std::{fs, sync::Mutex};

use color_eyre::{eyre::eyre, Result};
use pcsc::{Card, MAX_BUFFER_SIZE};

use crate::select_file::{select_file, File};

pub static DEBUG: Mutex<bool> = Mutex::new(false);

pub mod tlv;
pub mod data;

pub fn read_files(card: &Card, files: Vec<File>) {
    for file in files {
        println!("Selecting {file:?}");
        let data = match select_file(card, &file) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Encountered an error whilst reading {file:?}\n{e:?}");
                continue;
            }
        };
        println!("Finished reading {file:?}.");
        fs::write(format!("{file:?}.data"), &data).expect("Could not write file {file:?}");
    }
}

pub fn run_apdu(card: &Card, apdu: &Vec<u8>) -> Result<Vec<u8>> {
    if is_debug() {
        println!("Sending APDU: {apdu:?}");
    }

    let mut rapdu_buf = [0; MAX_BUFFER_SIZE];
    let response = match card.transmit(apdu, &mut rapdu_buf) {
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
        Err(eyre!("Got unsuccesful response from card: {response:X?}"))
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
    use pcsc::Card;

    use crate::{is_debug, run_apdu, tlv::FcpTemplate};

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

    #[derive(Debug)]
    pub enum File {
        FSOd,
        RegistrationA,
        RegistrationB,
        RegistrationC,
    }

    impl File {
        pub fn get_indetifier(&self) -> &[u8; 2] {
            match self {
                File::FSOd => &[0x00, 0x1D],
                File::RegistrationA => &[0xD0, 0x01],
                File::RegistrationB => &[0xD0, 0x11],
                File::RegistrationC => &[0xD0, 0x21],
            }
        }

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

    pub fn select_file(card: &Card, file: &File) -> Result<Vec<u8>> {
        let file_id = file.get_indetifier();
        let apdu = vec![CLA, INS, P1, P2, LC, file_id[0], file_id[1], LE];
        let response = run_apdu(card, &apdu)?;
        let fcp = FcpTemplate::parse(&response)?;
        // println!("Length of {file:?}: {length}");

        // println!("Reading {file:?}");
        let mut file = Vec::new();
        for i in (0..fcp.file_size).step_by(256) {
            let offset = &i.to_be_bytes();
            if is_debug() {
                println!("Reading offset: {i}; Hex: {offset:X?}");
            }
            let apdu = [0x00, 0xB0, offset[0], offset[1], 0x00].to_vec();
            let Ok(mut response) = run_apdu(card, &apdu) else {
                println!("Read beyond offset. Outputting what we have so far"); 
                return Ok(file);
            };
            file.append(&mut response);
        }

        Ok(file)
    }
}
