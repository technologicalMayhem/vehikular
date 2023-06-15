use gui::VehikularSettings;
use iced::Sandbox;

mod gui;
mod card_reading;
mod parsing; 
mod reader;

fn main() -> iced::Result {
    VehikularSettings::run(iced::Settings::default())
}
