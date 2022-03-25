use spreadget::Summary;

use crate::Options;

pub(crate) struct App {
    pub options: Options,
    pub summary: Summary,
    pub should_quit: bool,
}

impl App {
    pub fn new(options: Options) -> Self {
        App {
            options,
            summary: Summary::default(),
            should_quit: false,
        }
    }

    pub fn on_quit_key(&mut self) {
        self.should_quit = true;
    }

    pub fn on_new_summary(&mut self, summary: Summary) {
        self.summary = summary;
    }
}
