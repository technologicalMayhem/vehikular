use iced::{
    widget::{checkbox, column, button, row, text},
    Element, Sandbox, Settings,
};

struct VehikularSettings {
    readers: Vec<String>,
    cur_reader: usize,
    auto_upload: bool,
    auto_open: bool,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    ChangeConnection,
    ChangeReader,
    ToggleAutoUpload,
    ToggleAutoOpen,
    UploadCard,
    ViewCardLocal,
    ViewCardWeb
}

impl Sandbox for VehikularSettings {
    fn view(&self) -> Element<Message> {
        let connection_text = text("Currently connected to 10.10.0.69");
        let connection_edit = button("Change Server").on_press(Message::ChangeConnection);

        let connection = row![connection_text, connection_edit].spacing(5).align_items(iced::Alignment::Center);

        let reader_text = text("Using reader ");
        let reader_dropdown = iced::widget::pick_list(&self.readers, Some("Example Reader".to_string()), |_| Message::ChangeReader);

        let readers = row![reader_text, reader_dropdown].spacing(5).align_items(iced::Alignment::Center);

        let auto_upload = checkbox(
            "Automatically upload vehicle data when a card is inserted",
            self.auto_upload,
            |_| Message::ToggleAutoUpload,
        );
        let auto_open = checkbox(
            "Automatically open vehicle webpage when a card is inserted",
            self.auto_open,
            |_| Message::ToggleAutoOpen,
        );

        let manual_upload = button("Upload card content").on_press(Message::UploadCard);
        let view_local = button("View data locally").on_press(Message::ViewCardLocal);
        let view_web = button("View data on the web").on_press(Message::ViewCardWeb);

        let actions = row![manual_upload, view_local, view_web].spacing(5);

        column![connection, readers, auto_upload, auto_open, actions]
            .padding(10)
            .spacing(5)
            .into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::ToggleAutoUpload => self.auto_upload = !self.auto_upload,
            Message::ToggleAutoOpen => self.auto_open = !self.auto_open,
            Message::UploadCard => todo!(),
            Message::ViewCardLocal => todo!(),
            Message::ViewCardWeb => todo!(),
            Message::ChangeConnection => todo!(),
            Message::ChangeReader => todo!(),
        }
    }

    fn title(&self) -> String {
        "Vehikular Desktop".to_string()
    }

    type Message = Message;

    fn new() -> Self {
        VehikularSettings {
            readers: vec!["Example Reader".to_string()],
            cur_reader: 0,
            auto_upload: false,
            auto_open: false,
        }
    }
}

fn main() -> iced::Result {
    VehikularSettings::run(iced::Settings {
        window: iced::window::Settings { 
            ..Default::default() },
        ..Default::default()
    })
}
