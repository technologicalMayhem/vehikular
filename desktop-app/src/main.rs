use gui::VehikularSettings;
use iced::Application;

mod gui;
mod card_reading;
mod parsing; 
mod reader;

fn main() -> Result<(), iced::Error> {
    VehikularSettings::run(iced::Settings::default())
}
