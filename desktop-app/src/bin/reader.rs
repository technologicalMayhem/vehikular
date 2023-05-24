use std::env::args;

use color_eyre::Result;
use pcsc::{Context, Error, Protocols, Scope, ShareMode};

use card_reader::{
    read_files, run_apdu, select_file::File, DEBUG, SELECT_EVRC_APPLICATION,
    SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE,
};

fn main() -> Result<()> {
    for ele in args() {
        if ele == "--debug" {
            let mut bool = DEBUG.lock().expect("Failed to set debug to true.");
            *bool = true;
        }
    }

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
    let response = run_apdu(&card, &SELECT_EVRC_APPLICATION.to_vec())?;
    if response != SELECT_EVRC_APPLICATION_EXPECTED_RESPONSE {
        eprintln!("Got unexpected response: {response:X?}\nThis probably means that this is not a eVRC card.");
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
