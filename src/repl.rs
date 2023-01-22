use crate::source::SourceEvent;
use rustyline::{error::ReadlineError, Editor};
use std::sync::mpsc::Sender;

pub fn repl(events: Sender<SourceEvent>) {
    let mut editor = Editor::<()>::new().unwrap();
    loop {
        // TODO: Make real prompt once input/ouput timing is fixed.
        match editor.readline("") {
            Ok(line) => {
                events.send(SourceEvent::Repl(line)).unwrap();
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                // <Ctrl-C> or <Ctrl-D> to exit the REPl.
                events.send(SourceEvent::Exit).unwrap();
                break;
            }
            Err(error) => {
                println!("error: {:?}", error);
                break;
            }
        }
    }
}
