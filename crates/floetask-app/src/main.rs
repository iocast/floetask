use iced::widget::{center, text};
use iced::Element;

fn main() -> iced::Result {
    iced::application(Floetask::default, Floetask::update, Floetask::view)
        .title("floetask")
        .run()
}

#[derive(Default)]
struct Floetask;

#[derive(Debug, Clone)]
enum Message {}

impl Floetask {
    fn update(&mut self, message: Message) {
        match message {}
    }

    fn view(&self) -> Element<'_, Message> {
        center(text(format!("floetask {}", floetask_core::VERSION))).into()
    }
}
