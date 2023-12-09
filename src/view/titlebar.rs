use cursive::{View, XY};

pub struct Titlebar {
    title: String,
}

impl Titlebar {
    pub fn new(title: String) -> Self {
        Self { title }
    }

    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }
}

impl View for Titlebar {
    fn draw(&self, printer: &cursive::Printer) {
        printer.print(XY::zero(), self.title.as_str());
    }
}
