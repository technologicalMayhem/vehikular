use iced::{
    widget::{button, checkbox, column, pick_list, row, text, text_input},
    Alignment, Application, Command, Element,
};
use log::error;

use crate::reader::Reader;

pub struct VehikularSettings {
    address: String,
    auto_upload: bool,
    auto_open: bool,
    reader: Reader,
    selected_reader: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    AddressChanged(String),
    ChangeReader(String),
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
        let connection_edit =
            text_input("e.g. localhost:8000", "").on_input(Message::AddressChanged);
        let connection = row![connection_text, connection_ip, connection_edit]
            .spacing(5)
            .align_items(Alignment::Center);

        let reader_text = text("Using reader ");
        let reader_dropdown = pick_list(
            self.reader.get_readers(),
            self.selected_reader.clone(),
            Message::ChangeReader,
        );
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
            Message::ToggleAutoUpload => todo!(),
            Message::ToggleAutoOpen => todo!(),
            Message::UploadCard => {
                if let Some(reader) = &self.selected_reader {
                    match self.reader.process_reader(reader, &self.address) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("An error occured whilst processing the card: {err}")
                        }
                    }
                }
            }
            Message::ViewCardLocal => todo!(),
            Message::ViewCardWeb => todo!(),
            Message::AddressChanged(address) => self.address = address,
            Message::ChangeReader(reader) => self.selected_reader = Some(reader),
        };

        self.reader
            .update_readers()
            .expect("there was an issue whilst updating readers");

        Command::none()
    }

    fn title(&self) -> String {
        "Vehikular Desktop".to_string()
    }

    fn theme(&self) -> iced::Theme {
        iced::Theme::Dark
    }

    fn new(_flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (
            VehikularSettings {
                reader: Reader::new().expect("Could not create reader."),
                auto_upload: false,
                auto_open: false,
                address: String::new(),
                selected_reader: None,
            },
            Command::none(),
        )
    }
}
