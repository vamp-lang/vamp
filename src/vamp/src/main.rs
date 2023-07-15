use rustyline::error::ReadlineError;
use rustyline::{DefaultEditor, Result};
use vamp_eval::eval;
use vamp_sym::Interner;
use vamp_syntax::parser::parse_expr;

fn main() -> Result<()> {
    let mut editor = DefaultEditor::new()?;
    let mut interner = Interner::new();
    loop {
        let readline = editor.readline("> ");
        match readline {
            Ok(line) => {
                editor.add_history_entry(&line)?;
                match parse_expr(&line, &mut interner) {
                    Err(error) => {
                        eprintln!("error: {:?}", error);
                    }
                    Ok(expr) => match eval(&expr) {
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
