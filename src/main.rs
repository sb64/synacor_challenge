#[derive(Debug, Clone, Copy)]
enum Instruction {
    Halt,
    Set(Register, Literal),
    Push(Value),
    Pop(Location),
    Eq(Location, Value, Value),
    Gt(Location, Value, Value),
    Jmp(Location),
    Jt(Value, Location),
    Jf(Value, Location),
    Add(Location, Value, Value),
    Mult(Location, Value, Value),
    Mod(Location, Value, Value),
    And(Location, Value, Value),
    Or(Location, Value, Value),
    Not(Location, Value),
    Rmem(Location, Location),
    Wmem(Location, Location),
    Call(Location),
    Ret,
    Out(Value),
    In(Location),
    Noop,
}

#[derive(Debug, Clone, Copy)]
struct Register(usize);

#[derive(Debug, Clone, Copy)]
struct Literal(u16);

#[derive(Debug, Clone, Copy)]
enum Value {
    Literal(Literal),
    Register(Register),
}

#[derive(Debug, Clone, Copy)]
enum Location {
    Address(Address),
    Register(Register),
}

#[derive(Debug, Clone, Copy)]
struct Address(usize);

#[derive(Debug, Clone)]
struct Machine {
    mem: Box<[u16; 1 << 15]>,
    registers: Box<[u16; 8]>,
    stack: Vec<u16>,
}

impl Machine {
    fn new(program: &[u8]) -> Self {
        let mut iter = program
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]));

        Self {
            mem: Box::new(std::array::from_fn(|_| iter.next().unwrap_or(0))),
            registers: Box::new([0; 8]),
            stack: Vec::new(),
        }
    }

    fn run(&mut self) {
        todo!()
    }
}

fn main() {
    let program = std::fs::read("challenge.bin").expect("can't read file");
    let mut machine = Machine::new(&program);
    machine.run()
}
