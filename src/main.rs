mod eval;
mod parse;
mod repl;
mod source;
mod symbol;
mod tokens;
mod watch;
use eval::Environment;
use repl::repl;
use source::SourceEvent;
use std::{env, fs, io, path::Path, sync::mpsc, thread};
use watch::watch;
fn main() {
    let args: Vec<_> = env::args().collect();
    let (sender, receiver) = mpsc::channel();
    let root_path = match &args[1..] {
        [] => Path::new(".").to_owned(),
        [path] => Path::new(path).to_owned(),
        _ => {
            println!("usage: vamp [root_path]");
            return;
        }
    };

    // Source watcher
    thread::spawn({
        let sender = sender.clone();
        move || {
            if watch(&root_path, sender).is_err() {
                println!("error: could not watch filesystem events");
            }
        }
    });

    // REPL
    thread::spawn(move || {
        repl(sender);
    });

    // Handle all source events.
    let mut environment = Environment::new();
    for event in receiver {
        match event {
            SourceEvent::File(path) => {
                match fs::read_to_string(path) {
                    Ok(source) => {
                        if let Err(error) = environment.eval(&source) {
                            println!("error: {:?}", error);
                        }
                    }
                    Err(error) => {
                        if error.kind() == io::ErrorKind::NotFound {
                            // TODO: Delete definitions?
                        }
                    }
                }
            }
            SourceEvent::Repl(source) => match environment.eval(&source) {
                Ok(value) => println!("{}", value),
                Err(error) => println!("error: {:?}", error),
            },
            SourceEvent::Exit => {
                break;
            }
        }
    }
}
