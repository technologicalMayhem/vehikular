use gui::VehikularSettings;
use iced::Application;
use simplelog::Config;
use chrono::prelude::*;

mod card_reading;
mod gui;
mod parsing;
mod reader;

fn main() -> Result<(), iced::Error> {
    let time = Utc::now();
    let path = std::env::current_dir().expect("Could not get current dir").join(format!("Log-{time}-.txt"));
    let file = std::fs::File::create(path).expect("Could not create log file.");
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(log::LevelFilter::Debug, Config::default(), simplelog::TerminalMode::Mixed, simplelog::ColorChoice::Always),
        simplelog::WriteLogger::new(log::LevelFilter::Debug, simplelog::Config::default(), file),
    ]).expect("Could not create logging environtment.");
    VehikularSettings::run(iced::Settings::default())
}
