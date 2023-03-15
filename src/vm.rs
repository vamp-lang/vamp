#[repr(u8)]
pub enum Opcode {
    End,
    Push,
    Pop,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[repr(u8)]
pub enum Optype {
    U8,
    U16,
    U32,
    U64,
    I8,
    I16,
    I32,
    I64,
    F32,
    F64,
    Tuple,
}

struct Bytecode {
    bytes: *const u8,
    offset: usize,
}

impl<'a> Bytecode<'a> {
    fn new(bytes: *const u8) -> Self {
        Instructions { bytes, offset: 0 } 
    }

    fn next<T>() -> T {
        let size = std::mem::size_of<T>();
        let memory = unsafe {
            *std::mem::transmute::<*const u8, *const T>(self.bytes)
        };
        offset += size;
        memory
    }
}

pub struct Vm {
}

impl Vm {
    fn run(bytecode: Bytecode) {
        let stack = vec![];
        loop {
            match bytecode.next::<Opcode>() {
                Opcode::End => {
                    break,
                },
                Opcode::Push => {
                    match bytecode.next::<Optype>() {
                        Optype::U8 => {
                        },
                    }
                },
                Opcode::Pop => {
                    match bytecode.next::<Optype>() {

                    }
                },
                Opcode::Add => {
                },
                Opcode::Sub => {
                },
                Opcode::Mul => {
                },
                Opcode::Div => {
                },
                Opcode::Mod => {
                },
            }
        }
    }
}
