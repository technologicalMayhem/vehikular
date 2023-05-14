#![feature(iter_intersperse)]

use color_eyre::{eyre::eyre, Result};
use pcsc::{Card, Context, Error, Protocols, Scope, ShareMode, MAX_BUFFER_SIZE};

use crate::select_file::{select_file, File};

fn main() -> Result<()> {
    println!("{:?}", File::FSOd);

    // Establish a PC/SC context.
    let ctx = match Context::establish(Scope::User) {
        Ok(ctx) => ctx,
        Err(err) => {
            eprintln!("Failed to establish context: {err}");
            std::process::exit(1);
        }
    };

    // List available readers.
    let mut readers_buf = [0; 2048];
    let mut readers = match ctx.list_readers(&mut readers_buf) {
        Ok(readers) => readers,
        Err(err) => {
            eprintln!("Failed to list readers: {err}");
            std::process::exit(1);
        }
    };

    // Use the first reader.
    let Some(reader) = readers.next() else {
        println!("No readers are connected.");
        return Ok(());
    };
    println!("Using reader: {reader:?}");

    // Connect to the card.
    let card = match ctx.connect(reader, ShareMode::Shared, Protocols::ANY) {
        Ok(card) => card,
        Err(Error::NoSmartcard) => {
            println!("A smartcard is not present in the reader.");
            return Ok(());
        }
        Err(err) => {
            eprintln!("Failed to connect to card: {err}");
            std::process::exit(1);
        }
    };

    println!("Selecting eVRC application");
    let response = run_apdu(&card, &decode_hex(SELECT_EVRC_APPLICATION))?;
    if response != decode_hex(SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE) {
        eprintln!(
            "Got unexpected response: {}\nThis probably means that this is not a eVRD card.",
            encode_hex(&response)
        );
        std::process::exit(1);
    }

    read_files(
        &card,
        vec![
            File::FSOd,
            File::RegistrationA,
            File::RegistrationB,
            File::RegistrationC,
        ],
    );

    Ok(())
}

fn read_files(card: &Card, files: Vec<File>) {
    for file in files {
        println!("Selecting {file:?}");
        let file = match select_file(card, &file) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Encountered an error whilst reading {file:?}\n{e:?}");
                continue;
            }
        };
        println!(
            "Finished reading {file:?}. Here it is:\n{}",
            encode_hex(&file)
        );
    }
}

fn run_apdu(card: &Card, apdu: &Vec<u8>) -> Result<Vec<u8>> {
    println!("Sending APDU: {apdu:?}");
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
        Err(eyre!(
            "Got unsuccesful response from card: {}",
            encode_hex(response)
        ))
    }
}

fn decode_hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(3)
        .map(|i| {
            u8::from_str_radix(&s[i..i + 2], 16)
                .expect("Failed to parse APDU input command. Someone messed up the command :/")
        })
        .collect()
}

fn encode_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02x}"))
        .intersperse(" ".to_string())
        .collect()
}

fn is_sucessful(response: &[u8]) -> bool {
    let len = response.len();
    if len <= 1 {
        return false;
    }

    response[len - 2..len] == decode_hex("90 00")
}

const SELECT_EVRC_APPLICATION: &str = "00 A4 04 00 0B A0 00 00 04 56 45 56 52 2D 30 31 00";
const SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE: &str =
    "6F 0D 84 0B A0 00 00 04 56 45 56 52 2D 30 31 90 00";

mod select_file {
    use color_eyre::Result;
    use pcsc::Card;

    use crate::{decode_hex, encode_hex, run_apdu};

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
        fn get_indetifier(&self) -> &[u8; 2] {
            match self {
                File::FSOd => &[0x00, 0x1D],
                File::RegistrationA => &[0xD0, 0x01],
                File::RegistrationB => &[0xD0, 0x11],
                File::RegistrationC => &[0xD0, 0x21],
            }
        }
    }

    pub fn select_file(card: &Card, file: &File) -> Result<Vec<u8>> {
        let file_id = file.get_indetifier();
        let apdu = vec![CLA, INS, P1, P2, LC, file_id[0], file_id[1], LE];
        let response = run_apdu(card, &apdu)?;
        let first = response
            .get(8)
            .expect("Could not get first length byte")
            .to_owned();
        let second = response
            .get(9)
            .expect("Could not get second length byte")
            .to_owned();
        let length = u16::from_le_bytes([first, second]);
        println!("Length of {file:?}: {length}");

        println!("Reading {file:?}");
        let mut file = Vec::new();
        for i in (0..length).step_by(256) {
            let offset = encode_hex(&i.to_le_bytes());
            println!("Reading {offset}");
            let apdu = format!("00 B0 {offset} 00");
            let mut response = run_apdu(card, &decode_hex(&apdu))?;
            response.pop();
            response.pop();
            file.append(&mut response);
        }

        Ok(file)
    }
}
