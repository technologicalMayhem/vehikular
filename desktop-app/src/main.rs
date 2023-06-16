use chrono::prelude::*;
use gui::VehikularSettings;
use iced::Application;
use simplelog::ConfigBuilder;

mod card_reading;
mod gui;
mod parsing;
mod reader;

fn main() -> Result<(), iced::Error> {
    let time = Local::now().format("%Y-%m-%d %H-%M-%S");
    let path = std::env::current_dir()
        .expect("Could not get current dir")
        .join(format!("Log {time}.txt"));
    let file = std::fs::File::create(path).expect("Could not create log file.");
    let config = ConfigBuilder::new()
        .set_target_level(log::LevelFilter::Error)
        .add_filter_allow_str("desktop_app")
        .build();
    simplelog::CombinedLogger::init(vec![
        simplelog::TermLogger::new(
            log::LevelFilter::Debug,
            config.clone(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Always,
        ),
        simplelog::WriteLogger::new(log::LevelFilter::Debug, config, file),
    ])
    .expect("Could not create logging environtment.");
    VehikularSettings::run(iced::Settings::default())
}
