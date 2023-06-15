use std::{collections::HashSet};

use color_eyre::Result;
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
}

pub struct Reader {}

impl Reader {
    pub fn reader_thread() -> Result<(), Error> {
        let ctx = Context::establish(Scope::User)?;

        let mut readers_buf = [0; 2048];
        let mut reader_states = vec![
            // Listen for reader insertions/removals, if supported.
            ReaderState::new(PNP_NOTIFICATION(), State::UNAWARE),
        ];
        let mut have_been_read: HashSet<String> = HashSet::new();

        loop {
            // Remove dead readers.
            fn is_dead(rs: &ReaderState) -> bool {
                rs.event_state().intersects(State::UNKNOWN | State::IGNORE)
            }
            for rs in &reader_states {
                if is_dead(rs) {
                    println!("Removing {:?}", rs.name());
                    have_been_read.remove(&rs.name().to_string_lossy().to_string());
                }
            }
            reader_states.retain(|rs| !is_dead(rs));

            // Add new readers.
            let names = ctx
                .list_readers(&mut readers_buf)
                .expect("failed to list readers");
            for name in names {
                if !reader_states.iter().any(|rs| rs.name() == name) {
                    println!("Adding {name:?}");
                    reader_states.push(ReaderState::new(name, State::UNAWARE));
                }
            }

            // Update the view of the state to wait on.
            for rs in &mut reader_states {
                rs.sync_current_state();
            }

            // Wait until the state changes.
            ctx.get_status_change(None, &mut reader_states)?;

            // Print current state.
            println!();
            for rs in &reader_states {
                if rs.name() != PNP_NOTIFICATION()
                    && rs.event_state().contains(State::PRESENT)
                    && !have_been_read.contains(&rs.name().to_string_lossy().to_string())
                {
                    // Connect to the card.
                    let card = match ctx.connect(rs.name(), ShareMode::Shared, Protocols::ANY) {
                        Ok(card) => card,
                        Err(err) => {
                            eprintln!("Failed to connect to card: {err}");
                            std::process::exit(1);
                        }
                    };

                    println!("Found a card. Attempting read.");
                    match read_card(&card) {
                        Ok(registration) => {
                            println!("Read successful. Uploading.");
                            upload(&registration)?;
                        }
                        Err(err) => {
                            println!("Failed to read card. {err}");
                        }
                    };
                }
            }
        }
    }
}

fn upload(registration: &Registration) -> Result<(), reqwest::Error> {
    let client = reqwest::blocking::ClientBuilder::new().build()?;
    client.post("http://localhost:8000/registration").json(&registration).send()?;
    println!("Uploaded sucefully. Should be available under: http://localhost:8000/registration/{}", registration.registration_number);
    Ok(())
}