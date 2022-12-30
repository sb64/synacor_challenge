use std::{collections::VecDeque, fs::File, io::Write};

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

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "r{}", self.0)
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

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal(literal) => write!(f, "{literal}"),
            Value::LiteralAtRegister(register) => write!(f, "{register}"),
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

impl std::fmt::Display for Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
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

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Location::Address(address) => write!(f, "{address}"),
            Location::Register(register) => write!(f, "{register}"),
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

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#04x}", self.0)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Machine {
    mem: Vec<u16>,
    registers: Box<[u16; 8]>,
    stack: Vec<u16>,
    index: usize,
    stdin: VecDeque<u8>,
    #[serde(skip)]
    logger: Option<File>,
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
            logger: None,
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
        if self.index == 0x178b && self.registers[7] == 1 {
            println!("hacking...");
            self.mem[0x178b] = 18;
            self.registers[0] = 0x6;
            self.registers[7] = 0x6486;
        }
        let opcode = self.read_mem();
        Ok(match opcode {
            0 => {
                self.maybe_write_to_logger(format_args!("halt"), 1)?;

                Instruction::Halt
            }
            1 => {
                let register = self.read_register()?;
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;

                self.maybe_write_to_logger(format_args!("set  {register} {value}"), 3)?;

                Instruction::Set(register, literal)
            }
            2 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;

                self.maybe_write_to_logger(format_args!("push {value}"), 2)?;

                Instruction::Push(literal)
            }
            3 => {
                let location = self.read_location()?;

                self.maybe_write_to_logger(format_args!("pop  {location}"), 2)?;

                Instruction::Pop(location)
            }
            4 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("eq   {location} {leftv} {rightv}"), 4)?;

                Instruction::Eq(location, left, right)
            }
            5 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("gt   {location} {leftv} {rightv}"), 4)?;

                Instruction::Gt(location, left, right)
            }
            6 => {
                let location = self.read_location()?;
                let address = self.eval_location(location)?;

                self.maybe_write_to_logger(format_args!("jmp  {location}"), 2)?;

                Instruction::Jmp(address)
            }
            7 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                let location = self.read_location()?;
                let address = self.eval_location(location)?;

                self.maybe_write_to_logger(format_args!("jt   {value} {location}"), 3)?;

                Instruction::Jt(literal, address)
            }
            8 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;
                let location = self.read_location()?;
                let address = self.eval_location(location)?;

                self.maybe_write_to_logger(format_args!("jf   {value} {location}"), 3)?;

                Instruction::Jf(literal, address)
            }
            9 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("add  {location} {leftv} {rightv}"), 4)?;

                Instruction::Add(location, left, right)
            }
            10 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("mult {location} {leftv} {rightv}"), 4)?;

                Instruction::Mult(location, left, right)
            }
            11 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("mod  {location} {leftv} {rightv}"), 4)?;

                Instruction::Mod(location, left, right)
            }
            12 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("and  {location} {leftv} {rightv}"), 4)?;

                Instruction::And(location, left, right)
            }
            13 => {
                let location = self.read_location()?;
                let leftv = self.read_value()?;
                let left = self.eval_value(leftv)?;
                let rightv = self.read_value()?;
                let right = self.eval_value(rightv)?;

                self.maybe_write_to_logger(format_args!("or   {location} {leftv} {rightv}"), 4)?;

                Instruction::Or(location, left, right)
            }
            14 => {
                let location = self.read_location()?;
                let operandv = self.read_value()?;
                let operand = self.eval_value(operandv)?;

                self.maybe_write_to_logger(format_args!("not  {location} {operandv}"), 3)?;

                Instruction::Not(location, operand)
            }
            15 => {
                let dest = self.read_location()?;
                let srcl = self.read_location()?;
                let src = self.eval_location(srcl)?;

                self.maybe_write_to_logger(format_args!("rmem {dest} {srcl}"), 3)?;

                Instruction::Rmem(dest, src)
            }
            16 => {
                let destl = self.read_location()?;
                let dest = self.eval_location(destl)?;
                let srcv = self.read_value()?;
                let src = self.eval_value(srcv)?;

                self.maybe_write_to_logger(format_args!("wmem {destl} {srcv}"), 3)?;

                Instruction::Wmem(dest, src)
            }
            17 => {
                let location = self.read_location()?;
                let address = self.eval_location(location)?;

                self.maybe_write_to_logger(format_args!("call {location}"), 2)?;

                Instruction::Call(address)
            }
            18 => {
                self.maybe_write_to_logger(format_args!("ret "), 1)?;

                Instruction::Ret
            }
            19 => {
                let value = self.read_value()?;
                let literal = self.eval_value(value)?;

                self.maybe_write_to_logger(format_args!("out  {value}"), 2)?;

                Instruction::Out(literal)
            }
            20 => {
                let dest = self.read_location()?;

                self.maybe_write_to_logger(format_args!("in   {dest}"), 2)?;

                Instruction::In(dest)
            }
            21 => {
                self.maybe_write_to_logger(format_args!("noop"), 1)?;

                Instruction::Noop
            }
            _ => return Err(color_eyre::eyre::eyre!("got weird opcode: {opcode}")),
        })
    }

    fn redo_stdin(&mut self) {
        self.index -= 2;
        for ch in b"look\n".iter().rev().copied() {
            self.stdin.push_front(ch);
        }
    }

    fn read_stdin(&mut self) -> color_eyre::Result<Option<u16>> {
        match self.stdin.pop_front() {
            Some(raw) => Ok(Some(raw as u16)),
            None => {
                let mut line = String::new();

                let bytes_read = std::io::stdin()
                    .read_line(&mut line)
                    .wrap_err("read from stdin")?;
                if bytes_read == 0 {
                    return Err(color_eyre::eyre::eyre!("stdin has reached EOF"));
                }

                if line.starts_with("savestate") {
                    let (_, filename) = line.split_once(' ').wrap_err("get filename")?;
                    let filename = filename.trim();
                    std::fs::write(
                        filename,
                        serde_json::to_string(self).wrap_err("serialize state")?,
                    )
                    .wrap_err("save state")?;

                    std::process::exit(0);
                } else if line.starts_with("loadstate") {
                    let (_, filename) = line.split_once(' ').wrap_err("get filename")?;
                    let filename = filename.trim();
                    let deserialized = serde_json::from_str(
                        &std::fs::read_to_string(filename).wrap_err("load state")?,
                    )
                    .wrap_err("deserialize state")?;
                    *self = deserialized;

                    Ok(None)
                } else if line.starts_with("dumpregs") {
                    for (register, val) in self.registers.iter().copied().enumerate() {
                        println!("Register {register} = {val:#x}");
                    }

                    Ok(None)
                } else if line.starts_with("dumpreg") {
                    let (_, reg) = line.split_once(' ').wrap_err("get register")?;
                    let reg = reg
                        .trim()
                        .parse::<usize>()
                        .wrap_err("parse register into usize")?;
                    println!("Register {reg} = {:#x}", self.registers[reg]);

                    Ok(None)
                } else if line.starts_with("setreg") {
                    let mut iter = line.trim().splitn(3, ' ');
                    let _ = iter
                        .next()
                        .ok_or_else(|| color_eyre::eyre::eyre!("something sketchy's happening"))?;
                    let reg = iter
                        .next()
                        .ok_or_else(|| color_eyre::eyre::eyre!("get register"))?
                        .parse::<usize>()
                        .wrap_err("parse register into usize")?;
                    let val = iter
                        .next()
                        .ok_or_else(|| color_eyre::eyre::eyre!("get value"))?
                        .parse::<u16>()
                        .wrap_err("parse value into u16")?;
                    self.registers[reg] = val;

                    Ok(None)
                } else if line.starts_with("logfile") {
                    let (_, filename) = line.split_once(' ').wrap_err("get filename")?;
                    let filename = filename.trim();
                    let file = File::create(filename).wrap_err("create logfile")?;
                    self.logger = Some(file);

                    Ok(None)
                } else if line.starts_with("nolog") {
                    self.logger = None;

                    Ok(None)
                } else {
                    self.stdin.extend(
                        line.chars()
                            .filter_map(|ch| (ch != '\r').then_some(ch as u8)),
                    );
                    self.read_stdin()
                }
            }
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

    fn maybe_write_to_logger(
        &mut self,
        args: std::fmt::Arguments,
        index_offset: usize,
    ) -> color_eyre::Result<()> {
        if let Some(ref mut logger) = self.logger {
            writeln!(logger, "{:#06x}    {}", self.index - index_offset, args)
                .wrap_err("write to logger")?;
        }

        Ok(())
    }

    fn write_stdout(&mut self, raw: u16) {
        print!("{}", raw as u8 as char)
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
                Instruction::Out(literal) => self.write_stdout(literal.0),
                Instruction::In(location) => {
                    let raw = self.read_stdin()?;
                    match raw {
                        Some(raw) => self.write_to_location(location, raw),
                        None => self.redo_stdin(),
                    }
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

#[cfg(test)]
mod routine;

#[cfg(test)]
mod grid;
