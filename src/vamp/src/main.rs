use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebounceEventResult};
use rustc_hash::FxHashMap;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::{
    cell::RefCell,
    fs, io,
    path::{Path, PathBuf},
    rc::Rc,
    sync::mpsc::channel,
    time::Duration,
};
use toml;
use vamp_eval::{eval_module, eval_stmt, Mod, Scope, Value};
use vamp_sym::Interner;
use vamp_syntax::{parse_module, parse_stmt};
mod config;
use config::Config;

#[derive(Debug)]
enum Error {
    IoError(io::Error),
    SyntaxError(vamp_syntax::Error),
    RuntimeError(vamp_eval::Error),
}

struct Session {
    root: PathBuf,
    interner: Interner,
    scope: Rc<RefCell<Scope>>,
    ctx: Rc<RefCell<Scope>>,
    modules: FxHashMap<String, Mod>,
}

impl Session {
    fn new(root: PathBuf) -> Self {
        Self {
            root,
            interner: Interner::new(),
            scope: Rc::new(RefCell::new(Scope::new(None))),
            ctx: Rc::new(RefCell::new(Scope::new(None))),
            modules: FxHashMap::default(),
        }
    }

    fn load(&mut self, path: &Path, reload: bool) -> Result<(), Error> {
        let module_path = path
            .with_extension("")
            .components()
            .map(|c| c.as_os_str().to_str().unwrap())
            .collect::<Vec<_>>()
            .join(".");
        if !reload && self.modules.contains_key(&module_path) {
            return Ok(());
        }
        let source = fs::read_to_string(self.root.join(path)).map_err(Error::IoError)?;
        let module = parse_module(&source, &mut self.interner).map_err(Error::SyntaxError)?;
        for dep in module.deps.iter() {
            let mut dep_path = PathBuf::new();
            for segment in dep.path.segments.iter() {
                dep_path.push(self.interner.lookup(*segment));
            }
            self.load(&dep_path, false)?;
        }
        let module = eval_module(&module, self.scope.clone(), self.ctx.clone())
            .map_err(Error::RuntimeError)?;
        self.modules.insert(module_path.into(), module);
        Ok(())
    }

    fn eval_stmt(&mut self, stmt_source: &str) -> Result<Option<Value>, Error> {
        let stmt = parse_stmt(stmt_source, &mut self.interner).map_err(Error::SyntaxError)?;
        Ok(eval_stmt(&stmt, self.scope.clone(), self.ctx.clone()).map_err(Error::RuntimeError)?)
    }
}

pub enum SourceEvent {
    File(PathBuf),
    Repl(String),
    Exit,
}

fn main() {
    let config = match fs::read_to_string("vamp.toml") {
        Ok(config) => config,
        Err(_) => {
            eprintln!("error: this directory is not a vamp project yet");
            return;
        }
    };
    let config: Config = toml::from_str(&config).unwrap();
    let package = config.package;
    let root = Path::new(&package.root).canonicalize().unwrap();

    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_secs(1), None, {
        let tx = tx.clone();
        let root = root.clone();
        move |result: DebounceEventResult| match result {
            Ok(events) => {
                for event in events {
                    if event.path.extension() == Some("vamp".as_ref()) {
                        tx.send(SourceEvent::File(
                            event.path.strip_prefix(&root).unwrap().to_path_buf(),
                        ))
                        .unwrap();
                    }
                }
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("{:?}", error);
                }
            }
        }
    })
    .unwrap();
    debouncer
        .watcher()
        .watch(&root, RecursiveMode::Recursive)
        .unwrap();

    std::thread::spawn({
        let tx = tx.clone();
        move || {
            let mut editor = DefaultEditor::new().unwrap();
            loop {
                let readline = editor.readline("> ");
                match readline {
                    Ok(line) => {
                        editor.add_history_entry(&line).unwrap();
                        tx.send(SourceEvent::Repl(line)).unwrap();
                    }
                    Err(ReadlineError::Interrupted) => {
                        println!("Ctrl-C");
                        tx.send(SourceEvent::Exit).unwrap();
                        break;
                    }
                    Err(ReadlineError::Eof) => {
                        println!("Ctrl-D");
                        tx.send(SourceEvent::Exit).unwrap();
                        break;
                    }
                    Err(error) => {
                        eprintln!("error: {:?}", error);
                        break;
                    }
                }
            }
        }
    });

    let mut session = Session::new(root.clone());
    session.load(Path::new(&package.entry), false).unwrap();
    for event in rx {
        match event {
            SourceEvent::File(path) => session.load(&path, true).unwrap(),
            SourceEvent::Repl(line) => match session.eval_stmt(&line) {
                Ok(value) => {
                    println!("{:?}", value);
                }
                Err(error) => {
                    eprintln!("{:?}", error);
                }
            },
            SourceEvent::Exit => break,
        }
    }
}
