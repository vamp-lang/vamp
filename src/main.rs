mod ast;
mod eval;
mod lex;
mod parse;
mod repl;
mod source;
mod symbol;
mod watch;
use crate::parse::parse_statement;
use crate::repl::repl;
use crate::source::SourceEvent;
use crate::symbol::Interner;
use bumpalo::Bump;
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

    let mut interner = Interner::new();

    // Handle all source events.
    for event in receiver {
        match event {
            SourceEvent::File(path) => {
                match fs::read_to_string(path) {
                    Ok(source) => {
                        let arena = Bump::new();
                        let result = parse_statement(&source, &arena, &mut interner);
                        println!("{:?}", result);
                    }
                    Err(error) => {
                        if error.kind() == io::ErrorKind::NotFound {
                            // TODO: Delete definitions?
                        }
                    }
                }
            }
            SourceEvent::Repl(source) => {
                let arena = Bump::new();
                let result = parse_statement(&source, &arena, &mut interner);
                println!("{:?}", result);
            }
            SourceEvent::Exit => {
                break;
            }
        }
    }
}
