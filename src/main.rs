use std::collections::VecDeque;

use color_eyre::eyre::{Context, ContextCompat};

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Halt,
    Set(Register, Literal),
    Push(Literal),
    Pop(Location),
    Eq(Location, Literal, Literal),
    Gt(Location, Literal, Literal),
    Jmp(Address),
    Jt(Literal, Address),
    Jf(Literal, Address),
    Add(Location, Literal, Literal),
    Mult(Location, Literal, Literal),
    Mod(Location, Literal, Literal),
    And(Location, Literal, Literal),
    Or(Location, Literal, Literal),
    Not(Location, Literal),
    Rmem(Location, Address),
    Wmem(Address, Literal),
    Call(Address),
    Ret,
    Out(Literal),
    In(Location),
    Noop,
}

#[derive(Debug, Clone, Copy)]
struct Register(usize);

impl Register {
    fn new(register: u16) -> color_eyre::Result<Self> {
        if (32768..=32775).contains(&register) {
            Ok(Self(register as usize - 32768))
        } else {
            Err(color_eyre::eyre::eyre!("got weird register: {register}"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Value {
    Literal(Literal),
    LiteralAtRegister(Register),
}

impl Value {
    fn new(value: u16) -> color_eyre::Result<Self> {
        match value {
            0..=32767 => Ok(Value::Literal(Literal(value))),
            32768..=32775 => Ok(Value::LiteralAtRegister(Register(value as usize - 32768))),
            _ => Err(color_eyre::eyre::eyre!("got weird value: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Literal(u16);

impl Literal {
    fn new(literal: u16) -> color_eyre::Result<Self> {
        if (0..=32767).contains(&literal) {
            Ok(Self(literal))
        } else {
            Err(color_eyre::eyre::eyre!("got weird literal: {literal}"))
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum Location {
    Address(Address),
    Register(Register),
}

impl Location {
    fn new(location: u16) -> color_eyre::Result<Self> {
        match location {
            0..=32767 => Ok(Location::Address(Address(location as usize))),
            32768..=32775 => Ok(Location::Register(Register(location as usize - 32768))),
            _ => Err(color_eyre::eyre::eyre!("got weird location: {location}")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct Address(usize);

impl Address {
    fn new(address: u16) -> color_eyre::Result<Self> {
        if (0..=32767).contains(&address) {
            Ok(Self(address as usize))
        } else {
            Err(color_eyre::eyre::eyre!("got weird address: {address}"))
        }
    }
}

#[derive(Debug, Clone)]
struct Machine {
    mem: Vec<u16>,
    registers: Box<[u16; 8]>,
    stack: Vec<u16>,
    index: usize,
    stdin: VecDeque<u8>,
}

impl Machine {
    fn new(program: &[u8]) -> Self {
        let mut mem = vec![0; 1 << 15];
        for (i, val) in program
            .chunks_exact(2)
            .map(|bytes| u16::from_le_bytes([bytes[0], bytes[1]]))
            .enumerate()
        {
            mem[i] = val;
        }

        Self {
            mem,
            registers: Box::new([0; 8]),
            stack: Vec::new(),
            index: 0,
            stdin: VecDeque::new(),
        }
    }

    fn read_mem(&mut self) -> u16 {
        let mem = self.mem[self.index];
        self.index += 1;
        mem
    }

    fn read_register(&mut self) -> color_eyre::Result<Register> {
        let register = self.read_mem();
        Register::new(register)
    }

    fn read_value(&mut self) -> color_eyre::Result<Value> {
        let value = self.read_mem();
        Value::new(value)
    }

    fn read_location(&mut self) -> color_eyre::Result<Location> {
        let location = self.read_mem();
        Location::new(location)
    }

    fn read_instruction(&mut self) -> color_eyre::Result<Instruction> {
        let opcode = self.read_mem();
        Ok(match opcode {
            0 => Instruction::Halt,
            1 => {
                let register = self.read_register()?;
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                Instruction::Set(register, literal)
            }
            2 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                Instruction::Push(literal)
            }
            3 => {
                let location = self.read_location()?;
                Instruction::Pop(location)
            }
            4 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Eq(location, left, right)
            }
            5 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Gt(location, left, right)
            }
            6 => {
                let location = self.read_location()?;
                let address = self.eval_location(location)?;
                Instruction::Jmp(address)
            }
            7 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                let location = self.read_location()?;
                let address = self.eval_location(location)?;
                Instruction::Jt(literal, address)
            }
            8 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                let location = self.read_location()?;
                let address = self.eval_location(location)?;
                Instruction::Jf(literal, address)
            }
            9 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Add(location, left, right)
            }
            10 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Mult(location, left, right)
            }
            11 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Mod(location, left, right)
            }
            12 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::And(location, left, right)
            }
            13 => {
                let location = self.read_location()?;
                let left = self.read_value()?;
                let left = self.eval_value(left)?;
                let right = self.read_value()?;
                let right = self.eval_value(right)?;
                Instruction::Or(location, left, right)
            }
            14 => {
                let location = self.read_location()?;
                let operand = self.read_value()?;
                let operand = self.eval_value(operand)?;
                Instruction::Not(location, operand)
            }
            15 => {
                let dest = self.read_location()?;
                let src = self.read_location()?;
                let src = self.eval_location(src)?;
                Instruction::Rmem(dest, src)
            }
            16 => {
                let dest = self.read_location()?;
                let dest = self.eval_location(dest)?;
                let src = self.read_value()?;
                let src = self.eval_value(src)?;
                Instruction::Wmem(dest, src)
            }
            17 => {
                let location = self.read_location()?;
                let address = self.eval_location(location)?;
                Instruction::Call(address)
            }
            18 => Instruction::Ret,
            19 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                Instruction::Out(literal)
            }
            20 => {
                let dest = self.read_location()?;
                Instruction::In(dest)
            }
            21 => Instruction::Noop,
            _ => return Err(color_eyre::eyre::eyre!("got weird opcode: {opcode}")),
        })
    }

    fn read_stdin(&mut self) -> color_eyre::Result<u16> {
        match self.stdin.pop_front() {
            Some(raw) => Ok(raw as u16),
            None => Err(color_eyre::eyre::eyre!("read from stdin")),
        }
    }

    fn eval_register(&self, register: Register) -> u16 {
        self.registers[register.0]
    }

    fn eval_location(&self, location: Location) -> color_eyre::Result<Address> {
        match location {
            Location::Address(address) => Ok(address),
            Location::Register(register) => Address::new(self.eval_register(register)),
        }
    }

    fn eval_value(&self, value: Value) -> color_eyre::Result<Literal> {
        match value {
            Value::Literal(literal) => Ok(literal),
            Value::LiteralAtRegister(register) => Literal::new(self.eval_register(register)),
        }
    }

    fn write_to_location(&mut self, location: Location, raw: u16) {
        match location {
            Location::Address(address) => self.mem[address.0] = raw,
            Location::Register(register) => self.registers[register.0] = raw,
        }
    }

    fn write_stdin(&mut self, raw: u16) {
        let to_write = raw as u8;
        self.stdin.push_back(to_write);
        print!("{}", to_write as char)
    }

    fn pop_stack(&mut self) -> color_eyre::Result<u16> {
        self.stack.pop().wrap_err("pop stack")
    }

    fn run(&mut self) -> color_eyre::Result<()> {
        loop {
            match self.read_instruction()? {
                Instruction::Halt => return Ok(()),
                Instruction::Set(register, literal) => self.registers[register.0] = literal.0,
                Instruction::Push(literal) => self.stack.push(literal.0),
                Instruction::Pop(location) => {
                    let raw = self.pop_stack()?;
                    self.write_to_location(location, raw)
                }
                Instruction::Eq(location, left, right) => {
                    self.write_to_location(location, if left == right { 1 } else { 0 })
                }
                Instruction::Gt(location, left, right) => {
                    self.write_to_location(location, if left > right { 1 } else { 0 })
                }
                Instruction::Jmp(address) => self.index = address.0,
                Instruction::Jt(literal, address) => {
                    if literal.0 != 0 {
                        self.index = address.0
                    }
                }
                Instruction::Jf(literal, address) => {
                    if literal.0 == 0 {
                        self.index = address.0
                    }
                }
                Instruction::Add(dest, left, right) => {
                    let sum = (left.0 + right.0) % 32768;
                    self.write_to_location(dest, sum)
                }
                Instruction::Mult(dest, left, right) => {
                    let product = ((left.0 as u32 * right.0 as u32) % 32768) as u16;
                    self.write_to_location(dest, product)
                }
                Instruction::Mod(dest, left, right) => {
                    let rem = left.0 % right.0;
                    self.write_to_location(dest, rem)
                }
                Instruction::And(dest, left, right) => {
                    let anded = left.0 & right.0;
                    self.write_to_location(dest, anded)
                }
                Instruction::Or(dest, left, right) => {
                    let ored = left.0 | right.0;
                    self.write_to_location(dest, ored)
                }
                Instruction::Not(dest, operand) => {
                    let noted = !operand.0;
                    let noted = noted & 0x7fff;
                    self.write_to_location(dest, noted)
                }
                Instruction::Rmem(dest, src) => {
                    let mem = self.mem[src.0];
                    self.write_to_location(dest, mem)
                }
                Instruction::Wmem(dest, src) => self.mem[dest.0] = src.0,
                Instruction::Call(address) => {
                    self.stack.push(self.index as u16);
                    self.index = address.0
                }
                Instruction::Ret => {
                    let dest = self.pop_stack()? as usize;
                    self.index = dest
                }
                Instruction::Out(literal) => self.write_stdin(literal.0),
                Instruction::In(location) => {
                    let raw = self.read_stdin()?;
                    self.write_to_location(location, raw)
                }
                Instruction::Noop => {}
            }
        }
    }
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let program = std::fs::read("challenge.bin").wrap_err("read input file")?;
    let mut machine = Machine::new(&program);
    machine.run()?;

    Ok(())
}
