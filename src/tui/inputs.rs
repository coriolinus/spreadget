use crossterm::event::{self, KeyEvent};
use std::{
    sync::mpsc::{self, Receiver, RecvError, Sender},
    thread,
    time::Duration,
};

pub enum InputEvent {
    /// An input event occurred
    Input(KeyEvent),
    /// An event tick occurred
    Tick,
}

pub struct Events {
    rx: Receiver<InputEvent>,
    _tx: Sender<InputEvent>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Events {
        let (tx, rx) = mpsc::channel();

        let event_tx = tx.clone();
        thread::spawn(move || {
            loop {
                // poll for tick rate duration; if no event, send tick event
                if crossterm::event::poll(tick_rate).unwrap_or_default() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        event_tx.send(InputEvent::Input(key)).unwrap();
                    }
                }
            }
        });

        Events { _tx: tx, rx }
    }

    /// Attempts to read an event, blocking the current thread.
    pub fn next(&self) -> Result<InputEvent, RecvError> {
        self.rx.recv()
    }
}
