use crate::parse::parse;

pub struct Environment {}

impl Environment {
    pub fn new() -> Environment {
        Environment {}
    }

    pub fn eval(&mut self, namespace: &str, source: &str) {
        match parse(source) {
            Ok(expr) => {
                println!("parsed: {:?}", expr)
            }
            Err(error) => {
                println!("error: {:?}", error)
            }
        }
    }
}
