// RPtrPair: R25:R24 or R27:R26 or R29:R28 or R31:R31
// U6: Integer 0 <= x < 64
// Rd: Destination register R0-R31
// Rr: Register R0-R31


use std::convert::Into;
use std::ops::Add;


pub struct Assembler {
    pub buf: Vec<u8>
}

impl Assembler {
    pub fn new() -> Assembler{
        Assembler {
            buf: Vec::new()
        }
    }
}

pub fn relative(x: i32) -> Offset {
    assert!(x % 2 == 0);
    Offset::Relative(x / 2)
}
pub fn absolute(x: u32) -> Offset {
    Offset::Absolute(x)
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Register(u32);

pub const R0: Register = Register(0);
pub const R1: Register = Register(1);
pub const R2: Register = Register(2);
pub const R3: Register = Register(3);
pub const R4: Register = Register(4);
pub const R5: Register = Register(5);
pub const R6: Register = Register(6);
pub const R7: Register = Register(7);
pub const R8: Register = Register(8);
pub const R9: Register = Register(9);
pub const R10: Register = Register(10);
pub const R11: Register = Register(11);
pub const R12: Register = Register(12);
pub const R13: Register = Register(13);
pub const R14: Register = Register(14);
pub const R15: Register = Register(15);
pub const R16: Register = Register(16);
pub const R17: Register = Register(17);
pub const R18: Register = Register(18);
pub const R19: Register = Register(19);
pub const R20: Register = Register(20);
pub const R21: Register = Register(21);
pub const R22: Register = Register(22);
pub const R23: Register = Register(23);
pub const R24: Register = Register(24);
pub const R25: Register = Register(25);
pub const R26: Register = Register(26);
pub const R27: Register = Register(27);
pub const R28: Register = Register(28);
pub const R29: Register = Register(29);
pub const R30: Register = Register(30);
pub const R31: Register = Register(31);

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RegisterPair(pub Register, pub Register);

pub const X: RegisterPair = RegisterPair(R27, R26);
pub const Y: RegisterPair = RegisterPair(R29, R28);
pub const Z: RegisterPair = RegisterPair(R31, R30);

enum Direction {
    PreDecrement,
    NoChange,
    PostIncrement,
}

pub struct DirectionalRegisterPair {
    pair: RegisterPair,
    direction: Direction,
}


pub struct OffsetRegisterPair {
    pair: RegisterPair,
    offset: u8,
}


impl RegisterPair {
    pub fn post_increment(self) -> DirectionalRegisterPair {
        DirectionalRegisterPair {
            pair: self,
            direction: Direction::PostIncrement,
        }
    }
    pub fn pre_decrement(self) -> DirectionalRegisterPair {
        DirectionalRegisterPair {
            pair: self,
            direction: Direction::PreDecrement,
        }
    }
}

impl Into<DirectionalRegisterPair> for RegisterPair {
    fn into(self) -> DirectionalRegisterPair {
        DirectionalRegisterPair{
            pair: self,
            direction: Direction::NoChange,
        }
    }
}

impl Add<u8> for RegisterPair {
    type Output = OffsetRegisterPair;
    
    fn add(self, rhs: u8) -> OffsetRegisterPair {
        OffsetRegisterPair{
            pair: self,
            offset: rhs,
        }
    }
}

#[derive(Copy, Clone)]
pub enum Offset {
    Absolute(u32),
    Relative(i32),
}

#[derive(Copy, Clone)]
enum Arg {
    Register(Register),
    RegisterPair(RegisterPair),
    U32(u32),
    U8(u8),
    I32(i32),
}

impl Into<Arg> for Register {
    fn into(self) -> Arg {
        Arg::Register(self)
    }
}

impl Into<Arg> for RegisterPair {
    fn into(self) -> Arg {
        Arg::RegisterPair(self)
    }
}

impl From<u32> for Arg {
    fn from(x: u32) -> Arg {
        Arg::U32(x)
    }
}

impl From<i32> for Arg {
    fn from(x: i32) -> Arg {
        Arg::I32(x)
    }
}
impl From<u8> for Arg {
    fn from(x: u8) -> Arg {
        Arg::U8(x)
    }
}

#[derive(Copy, Clone, Debug)]
struct ArgConsumer {
    value: u32
}

impl ArgConsumer {
    fn new(arg: Arg, bit_count: usize) -> ArgConsumer{
        ArgConsumer{
            value: match arg {
                Arg::Register(r) => {
                    match bit_count {
                        10 => {
                            // This assumes a bit pattern xy xxxx yyyy
                            assert!(r.0 < 32);
                            (r.0 << 4) | ((r.0 & 16) << 9) | (r.0 & 0x0f)
                        }
                        5 => {
                            assert!(r.0 < 32);
                            r.0
                        }
                        4 => {
                            assert!(16 <= r.0 && r.0 < 32);
                            r.0 & 15
                        }
                        3 => {
                            assert!(16 <= r.0 && r.0 < 24);
                            r.0 & 7
                        }
                        _ => {
                            panic!("unrecognized number of register bits: {}", bit_count);
                        }
                    }
                },
                Arg::RegisterPair(RegisterPair(high, low)) => {
                    if high.0 != low.0+1 {
                        panic!("Register pair must be consecutive HI,LO registers");
                    }
                    if low.0 % 2 == 1 {
                        panic!("Low register in register pair must be an even-numbered register");
                    }
                    let min_representable = 32 - 2 * (1 << bit_count);
                    if low.0 < min_representable {
                        panic!("Register pair out of representable range. Must be at least R{}", min_representable);
                    }
                    low.0 / 2
                }
                Arg::U32(x) => {
                    x
                }
                Arg::U8(x) => {
                    x as u32
                }
                Arg::I32(x) => {
                    // TODO: Validate that it fits in the big count?
                    x as u32
                }
            }
        }
    }
    
    fn consume_bit(&mut self) -> u32 {
        let ret = self.value & 1;
        self.value = self.value >> 1;
        //println!("consumed {}, has {} left", ret, self.value);
        return ret;
    }
}

impl Assembler {
    fn encode(&mut self, args: &[(Arg, u8)], format: &[u8]) {
        let mut occurrence_count = [0; 256];
        for format_byte in format {
            occurrence_count[*format_byte as usize] += 1;
        }
        
        let mut arg_consumers: [Option<ArgConsumer>; 256] = [None; 256];
        for &(arg, format_byte) in args {
            arg_consumers[format_byte as usize] = Some(ArgConsumer::new(arg, occurrence_count[format_byte as usize]));
        }
        
        let mut result: u32 = 0;
        let mut result_bits: usize = 0;
        
        for format_byte in format.iter().rev() {
            //println!("{}", format_byte);
            let bit = match *format_byte {
                b' ' => { continue },
                b'0' => 0,
                b'1' => 1,
                ch => arg_consumers[ch as usize].as_mut().expect("Format character has no argument").consume_bit()
            };
            result = result | (bit << result_bits);
            result_bits += 1;
        }
        
        assert!(result_bits % 16 == 0);
        while result_bits > 0 {
            self.buf.push(((result >> (result_bits - 16)) & 0xff) as u8);
            self.buf.push(((result >> (result_bits - 8)) & 0xff) as u8);
            result_bits -= 16;
        }
    }
    
    fn resolve_absolute_offset(&self, offset: Offset) -> u32 {
        match offset {
            Offset::Absolute(x) => x,
            Offset::Relative(x) => (self.buf.len() as i32 + x) as u32
        }
    }
    fn resolve_absolute_offset_doubles(&self, offset: Offset) -> u32 {
        let offset = self.resolve_absolute_offset(offset);
        assert!(offset % 2 == 0, "offset must be even");
        offset / 2
    }
    fn resolve_relative_offset(&self, offset: Offset) -> i32 {
        (self.resolve_absolute_offset(offset) as i32) - (self.buf.len() as i32)
    }
    
    pub fn lds(&mut self, d: Register, k: Offset) {
        let addr = self.resolve_absolute_offset(k);
        //println!("resolved to {}", addr);
        if addr < 128 {
            self.lds_7(d, k);
        } else {
            self.lds_16(d, k);
        }
    }
    
    pub fn ld<R: Into<DirectionalRegisterPair>>(&mut self, d: Register, r: R) {
        let r: DirectionalRegisterPair = r.into();
        let template = match (r.pair, r.direction) {
            (x, Direction::NoChange) if x == X => b"1001 000d dddd 1100",
            (x, Direction::PostIncrement) if x == X => b"1001 000d dddd 1101",
            (x, Direction::PreDecrement) if x == X => b"1001 000d dddd 1110",
            (y, Direction::NoChange) if y == Y => b"1000 000d dddd 1000",
            (y, Direction::PostIncrement) if y == Y => b"1001 000d dddd 1001",
            (y, Direction::PreDecrement) if y == Y => b"1001 000d dddd 1010",
            (z, Direction::NoChange) if z == Z => b"1000 000d dddd 0000",
            (z, Direction::PostIncrement) if z == Z => b"1001 000d dddd 0001",
            (z, Direction::PreDecrement) if z == Z => b"1001 000d dddd 0010",
            _ => panic!("Invalid LD arguments")
        };
        
        self.encode(&[(d.into(), b'd')][..], template)
    }
    
    pub fn ldd(&mut self, d: Register, r: OffsetRegisterPair) {
        let template = 
            if r.pair == Y {
                b"10q0 qq0d dddd 1qqq"
            } else if r.pair == Z {
                b"10q0 qq0d dddd 0qqq"
            } else {
                panic!("Invalid pointer for LDD");
            };
        self.encode(&[(d.into(), b'd'), (r.offset.into(), b'q')][..], template)
    }
    
    pub fn st<D: Into<DirectionalRegisterPair>>(&mut self, d: D, r: Register) {
        let d: DirectionalRegisterPair = d.into();
        let template = match (d.pair, d.direction) {
            (x, Direction::NoChange) if x == X => b"1001 001r rrrr 1100",
            (x, Direction::PostIncrement) if x == X => b"1001 001r rrrr 1101",
            (x, Direction::PreDecrement) if x == X => b"1001 001r rrrr 1110",
            (y, Direction::NoChange) if y == Y => b"1000 001r rrrr 1000",
            (y, Direction::PostIncrement) if y == Y => b"1001 001r rrrr 1001",
            (y, Direction::PreDecrement) if y == Y => b"1001 001r rrrr 1010",
            (z, Direction::NoChange) if z == Z => b"1000 001r rrrr 0000",
            (z, Direction::PostIncrement) if z == Z => b"1001 001r rrrr 0001",
            (z, Direction::PreDecrement) if z == Z => b"1001 001r rrrr 0010",
            _ => panic!("Invalid ST arguments")
        };
        
        self.encode(&[(r.into(), b'r')][..], template)
    }
    
    pub fn std(&mut self, d: OffsetRegisterPair, r: Register) {
        let template = 
            if d.pair == Y {
                b"10q0 qq1r rrrr 1qqq"
            } else if d.pair == Z {
                b"10q0 qq1r rrrr 0qqq"
            } else {
                panic!("Invalid pointer for STD");
            };
        self.encode(&[(r.into(), b'r'), (d.offset.into(), b'q')][..], template)
    }
    
    pub fn lpm<R: Into<DirectionalRegisterPair>>(&mut self, d: Register, r: R) {
        let r: DirectionalRegisterPair = r.into();
        let template = match (r.pair, r.direction) {
            (z, Direction::NoChange) if z == Z => b"1001 000d dddd 0100",
            (z, Direction::PostIncrement) if z == Z => b"1001 000d dddd 0101",
            _ => panic!("Invalid LPM arguments")
        };
        
        self.encode(&[(d.into(), b'd')][..], template)
    }
    
    pub fn elpm<R: Into<DirectionalRegisterPair>>(&mut self, d: Register, r: R) {
        let r: DirectionalRegisterPair = r.into();
        let template = match (r.pair, r.direction) {
            (z, Direction::NoChange) if z == Z => b"1001 000d dddd 0110",
            (z, Direction::PostIncrement) if z == Z => b"1001 000d dddd 0111",
            _ => panic!("Invalid ELPM arguments")
        };
        
        self.encode(&[(d.into(), b'd')][..], template)
    }
}


include!(concat!(env!("OUT_DIR"), "/ops.rs"));

