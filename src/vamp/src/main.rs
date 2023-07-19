use notify::{RecursiveMode, Watcher};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use vamp_eval::{eval_stmt, Scope};
use vamp_sym::Interner;
use vamp_syntax::parser::parse_stmt;

struct Workspace {
    root_path: PathBuf,
    watcher: Watcher,
}

impl Workspace {
    fn new(root_path: PathBuf) -> Self {
        let debouncer = new_debouncer(
            Duration::from_secs(1),
            None,
            |result: DebouncedEventResult| match result {

            }
        );
        Self {
            root_path,

        }
    }

    fn reload(&self, path: &Path) {}

    fn watch(&self) {
        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            None,
            |result: DebouncedEventResult| match result {
                Ok(events) => {
                    if event.path.extension() == Some("vamp".as_ref()) {
                        self.reload(&event.path);
                    }
                }
                Err(errors) => {
                    for error in errors {
                        eprintln!("error: {:?}", error);
                    }
                }
            },
        );
        let watcher = debouncer.watcher();
    }
}

fn main() {
    let root_path = Path::new(".");
    let workspace = Workspace::new(root_path.to_owned());
    let mut debouncer = new_debouncer(
        Duration::from_secs(1),
        None,
        |event: DebounceEventResult| match event {
            Ok(events) => {
                for event in events {
                    if event.path.extension() == Some("vamp".as_ref()) {
                        reload_file(&event.path);
                    }
                }
            }
            Err(errors) => {
                eprintln!("error: {:?}", errors);
            }
        },
    )
    .unwrap();
    let watcher = debouncer.watcher();
    watcher.watch(&root_path, RecursiveMode::Recursive).unwrap();

    let mut editor = DefaultEditor::new().unwrap();
    let mut interner = Interner::new();
    let mut scope = Scope::default();
    loop {
        let readline = editor.readline("> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(&line).unwrap();
                match parse_stmt(&line, &mut interner) {
                    Err(error) => {
                        eprintln!("error: {:?}", error);
                    }
                    Ok(expr) => match eval_stmt(&expr, &mut scope) {
                        Err(error) => {
                            eprintln!("error: {:?}", error);
                        }
                        Ok(value) => {
                            println!("{:?}", value);
                        }
                    },
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            }
            Err(error) => {
                eprintln!("error: {:?}", error);
                break;
            }
        }
    }
}
