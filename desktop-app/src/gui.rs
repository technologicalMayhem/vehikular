use std::borrow::ToOwned;

use iced::{
    widget::{button, checkbox, column, row, text},
    Alignment, Element, Application, Command,
};
use iced::Sandbox;

use crate::reader::Reader;

pub struct VehikularSettings {
    auto_upload: bool,
    auto_open: bool,
    reader: Reader,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    ChangeConnection,
    ChangeReader,
    ToggleAutoUpload,
    ToggleAutoOpen,
    UploadCard,
    ViewCardLocal,
    ViewCardWeb,
}

impl Application for VehikularSettings {
    type Message = Message;
    type Executor = iced::executor::Default;
    type Theme = iced::Theme;
    type Flags = ();

    fn view(&self) -> Element<Message> {
        let connection_text = text("Currently connected to");
        let connection_ip = text("10.10.0.69");
        let connection_edit = button("Change Server").on_press(Message::ChangeConnection);
        let connection = row![connection_text, connection_ip, connection_edit]
            .spacing(5)
            .align_items(Alignment::Center);

        let reader_text = text("Using reader ");
        let selected = self.reader.readers.first().map(ToOwned::to_owned);
        let reader_dropdown =
            iced::widget::pick_list(&self.reader.readers, selected, |_| Message::ChangeReader);
        let readers = row![reader_text, reader_dropdown]
            .spacing(5)
            .align_items(Alignment::Center);

        let auto_text = text("When a card is inserted");
        let auto_upload = checkbox(
            "Automatically upload vehicle data",
            self.auto_upload,
            |_| Message::ToggleAutoUpload,
        );
        let auto_open = checkbox("Open vehicle webpage", self.auto_open, |_| {
            Message::ToggleAutoOpen
        });
        let auto = column![auto_text, auto_upload, auto_open].spacing(5);

        let manual_upload = button("Upload card content").on_press(Message::UploadCard);
        let view_local = button("View data locally").on_press(Message::ViewCardLocal);
        let view_web = button("View data on the web").on_press(Message::ViewCardWeb);

        let actions = row![manual_upload, view_local, view_web].spacing(5);

        column![connection, readers, auto, actions]
            .padding(10)
            .spacing(10)
            .into()
    }

    fn update(&mut self, message: Message) -> iced::Command<Message> {
        match message {
            Message::ToggleAutoUpload => self.auto_upload = !self.auto_upload,
            Message::ToggleAutoOpen => self.auto_open = !self.auto_open,
            Message::UploadCard => todo!(),
            Message::ViewCardLocal => todo!(),
            Message::ViewCardWeb => todo!(),
            Message::ChangeConnection => todo!(),
            Message::ChangeReader => todo!(),
        };

        Command::none()
    }

    fn title(&self) -> String {
        "Vehikular Desktop".to_string()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }


    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (VehikularSettings {
            reader: Reader::new(),
            auto_upload: false,
            auto_open: false,
        }, Command::none())
    }
}