use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use vamp_eval::{eval_stmt, Scope};
use vamp_sym::Interner;
use vamp_syntax::parser::parse_stmt;

fn main() -> Result<()> {
    let mut editor = DefaultEditor::new()?;
    let mut interner = Interner::new();
    let mut scope = Scope::default();
    loop {
        let readline = editor.readline("> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(&line)?;
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
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(_) => break,
        }
    }
    Ok(())
}
