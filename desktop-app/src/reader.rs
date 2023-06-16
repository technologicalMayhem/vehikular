use std::{collections::HashSet, time::Duration};

use color_eyre::Result;
use log::{error, info};
use pcsc::{Context, Protocols, ReaderState, Scope, ShareMode, State, PNP_NOTIFICATION};
use thiserror::Error;

use crate::card_reading::read_card;
use shared::data::Registration;

#[derive(Debug, Error)]
pub enum Error {
    #[error("The underlying card reader library returned an error: {0}")]
    Pcsc(#[from] pcsc::Error),
    #[error("An error occured when interacting with the server: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Could not connect to card.")]
    ConectionFailure,
    #[error("Reader indicates no card found.")]
    CardNotFound,
    #[error("Attemted to use PNP notifcation as a reader")]
    PnpNotficationAsReader,
    #[error("The selected reader could not be found. Did you disconnect it?")]
    ReaderNotFound,
}

pub struct Reader {
    ctx: Context,
    reader_states: Vec<ReaderState>,
    have_been_read: HashSet<String>,
}

impl Reader {
    pub fn new() -> Result<Self, Error> {
        Ok(Reader {
            ctx: Context::establish(Scope::User)?,
            reader_states: vec![
                // Listen for reader insertions/removals, if supported.
                ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
            ],
            have_been_read: HashSet::new(),
        })
    }

    /// Tests if a reader is dead.
    fn is_dead(rs: &ReaderState) -> bool {
        rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
    }

    pub fn update_readers(&mut self) -> Result<(), Error> {
        let mut readers_buf = [0; 2048];
        // Remove dead readers.

        for rs in &self.reader_states {
            if Self::is_dead(rs) {
                println!("Removing {:?}", rs.name());
                self.have_been_read
                    .remove(&rs.name().to_string_lossy().to_string());
            }
        }
        self.reader_states.retain(|rs| !Self::is_dead(rs));

        // Add new readers.
        let names = self.ctx.list_readers(&mut readers_buf)?;
        for name in names {
            if !self.reader_states.iter().any(|rs| rs.name() == name) {
                println!("Adding {name:?}");
                self.reader_states
                    .push(ReaderState::new(name, State::UNAWARE));
            }
        }

        // Update the view of the state to wait on.
        for rs in &mut self.reader_states {
            rs.sync_current_state();
        }

        // Wait until the state changes.
        match self.ctx.get_status_change(Some(Duration::from_millis(10)), &mut self.reader_states) {
            Err(pcsc::Error::Timeout) | Ok(_) => {},
            Err(err) => Err(err)?
        };

        let readers = self
            .reader_states
            .iter()
            .map(|rs| format!("Name: {}; State: {:?};", rs.name().to_string_lossy(), rs.event_state()))
            .collect::<Vec<String>>()
            .join("\n         ");
        info!("Readers:\n{readers}");

        Ok(())
    }

    pub fn get_readers(&self) -> Vec<String> {
        self.reader_states
            .iter()
            .filter_map(|rs| {
                if rs.name() == PNP_NOTIFICATION() {
                    None
                } else {
                    Some(rs.name().to_string_lossy().to_string())
                }
            })
            .collect()
    }

    pub fn process_reader(&self, reader: &str, upload_address: &str) -> Result<(), Error> {
        let Some(reader) = self
            .reader_states
            .iter()
            .position(|rs| rs.name().to_string_lossy() == reader)
            .and_then(|index| self.reader_states.get(index))
        else {
            Err(Error::ReaderNotFound)?
        };
        info!("Using reader: {}", reader.name().to_string_lossy());

        if reader.name() == PNP_NOTIFICATION() {
            Err(Error::PnpNotficationAsReader)?;
        }

        if !reader.event_state().contains(State::PRESENT) {
            Err(Error::CardNotFound)?;
        }
        // Connect to the card.
        let card = match self
            .ctx
            .connect(reader.name(), ShareMode::Shared, Protocols::ANY)
        {
            Ok(card) => card,
            Err(err) => {
                error!("Failed to connect to card: {err}");
                Err(Error::ConectionFailure)?
            }
        };

        info!("Found a card. Attempting read.");
        match read_card(&card) {
            Ok(registration) => {
                info!("Read successful. Uploading.");
                upload(&registration, upload_address)?;
            }
            Err(err) => {
                error!("Failed to read card. {err}");
            }
        }

        Ok(())
    }
}

fn upload(registration: &Registration, upload_address: &str) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    client
        .post(format!("http://{upload_address}/registration"))
        .json(&registration)
        .send()?;
    println!(
        "Uploaded sucefully. Should be available under: http://{upload_address}/registration/{}",
        registration.registration_number
    );
    Ok(())
}
