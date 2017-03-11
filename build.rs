
use std::env;
use std::path::Path;
use std::fs::File;
use std::io::Write;

type InstructionSpec = (&'static str, &'static [Arg], &'static str);

#[derive(Copy, Clone, Debug)]
enum Arg {
    Register(char),
    RegisterPair(char),
    Unsigned(char),
    RelativeOffset(char),
    AbsoluteOffset(char),
    AbsoluteOffsetDoubles(char), // measured by 16-bit intervals
    ImplicitZ,
    
}

impl Arg {
    fn name(&self) -> String {
        self.format_char().to_string().to_lowercase()
    }
    
    fn format_char(&self) -> char {
        match self { 
            &Arg::Register(c) => c,
            &Arg::RegisterPair(c) => c,
            &Arg::Unsigned(c) => c,
            &Arg::RelativeOffset(c) => c,
            &Arg::AbsoluteOffset(c) => c,
            &Arg::AbsoluteOffsetDoubles(c) => c,
            &Arg::ImplicitZ => 'z',
        }
    }
    
    fn type_str(&self) -> String {
        match self {
            &Arg::Register(_) => "Register".to_string(),
            &Arg::RegisterPair(_) => "RegisterPair".to_string(),
            &Arg::Unsigned(_) => "u32".to_string(),
            &Arg::RelativeOffset(_) => "Offset".to_string(),
            &Arg::AbsoluteOffset(_) => "Offset".to_string(),
            &Arg::AbsoluteOffsetDoubles(_) => "Offset".to_string(),
            &Arg::ImplicitZ => "RegisterPair".to_string(),
        }
    }
}

const RD: Arg = Arg::Register('d');
const RR: Arg = Arg::Register('r');
const RD_PAIR: Arg = Arg::RegisterPair('d');
const RR_PAIR: Arg = Arg::RegisterPair('r');
const K: Arg = Arg::Unsigned('K');
const S: Arg = Arg::Unsigned('s');
const B: Arg = Arg::Unsigned('b');
const OFFSET: Arg = Arg::RelativeOffset('k');
const ABSOLUTE_OFFSET: Arg = Arg::AbsoluteOffset('k');
const ABSOLUTE_OFFSET_DOUBLES: Arg = Arg::AbsoluteOffsetDoubles('k');
const A: Arg = Arg::Unsigned('A');

static INSTRUCTIONS: [InstructionSpec; 117] = [
    ("adc", &[RD, RR], "0001 11rd dddd rrrr"),
    ("add", &[RD, RR], "0000 11rd dddd rrrr"),
    ("adiw", &[RD_PAIR, K], "1001 0110 KKdd KKKK"),
    ("and", &[RD, RR], "0010 00rd dddd rrrr"),
    ("andi", &[RD, K], "0111 KKKK dddd KKKK"),
    ("asr", &[RD], "1001 010d dddd 0101"),
    ("bclr", &[S], "1001 0100 1sss 1000"), // TODO: Tests?
    ("bld", &[RD, B], "1111 100d dddd 0bbb"),

    ("brbc", &[S, OFFSET], "1111 01kk kkkk ksss"),
    ("brbs", &[S, OFFSET], "1111 00kk kkkk ksss"),
    ("brcc", &[OFFSET], "1111 01kk kkkk k000"),
    ("brcs", &[OFFSET], "1111 00kk kkkk k000"),
    ("break_", &[], "1001 0101 1001 1000"),
    ("breq", &[OFFSET], "1111 00kk kkkk k001"),
    ("brge", &[OFFSET], "1111 01kk kkkk k100"),
    ("brhc", &[OFFSET], "1111 01kk kkkk k101"),
    ("brhs", &[OFFSET], "1111 00kk kkkk k101"),
    ("brid", &[OFFSET], "1111 01kk kkkk k111"),
    ("brie", &[OFFSET], "1111 00kk kkkk k111"),
    ("brlo", &[OFFSET], "1111 00kk kkkk k000"),
    ("brlt", &[OFFSET], "1111 00kk kkkk k100"),
    ("brmi", &[OFFSET], "1111 00kk kkkk k010"),
    ("brne", &[OFFSET], "1111 01kk kkkk k001"),
    ("brpl", &[OFFSET], "1111 01kk kkkk k010"),
    ("brsh", &[OFFSET], "1111 01kk kkkk k000"),
    ("brtc", &[OFFSET], "1111 01kk kkkk k110"),
    ("brts", &[OFFSET], "1111 00kk kkkk k110"),
    ("brvc", &[OFFSET], "1111 01kk kkkk k011"),
    ("brvs", &[OFFSET], "1111 00kk kkkk k011"),

    
    ("bset", &[S], "1001 0100 0sss 1000"),
    ("bst", &[RD, B], "1111 101d dddd 0bbb"),
    ("call", &[ABSOLUTE_OFFSET_DOUBLES], "1001 010k kkkk 111k kkkk kkkk kkkk kkkk"),
    ("cbi", &[A, B], "1001 1000 AAAA Abbb"),
    ("cbr", &[RD, K], "0111 KKKK dddd KKKK"), // TODO: test this
    ("clc", &[], "1001 0100 1000 1000"),
    ("clh", &[], "1001 0100 1101 1000"),
    ("cli", &[], "1001 0100 1111 1000"),
    ("cln", &[], "1001 0100 1010 1000"),
    ("clr", &[RD], "0010 01dd dddd dddd"), // EOR RD,RD
    ("cls", &[], "1001 0100 1100 1000"),
    ("clt", &[], "1001 0100 1110 1000"),
    ("clv", &[], "1001 0100 1011 1000"),
    ("clz", &[], "1001 0100 1001 1000"),
    ("com", &[RD], "1001 010d dddd 0000"),
    ("cp", &[RD, RR], "0001 01rd dddd rrrr"),
    ("cpc", &[RD, RR], "0000 01rd dddd rrrr"),
    ("cpi", &[RD, K], "0011 KKKK dddd KKKK"),
    ("cpse", &[RD, RR], "0001 00rd dddd rrrr"),
    ("dec", &[RD], "1001 010d dddd 1010"),
    ("des", &[K], "1001 0100 KKKK 1011"),
    ("eicall", &[], "1001 0101 0001 1001"),
    ("eijmp", &[], "1001 0100 0001 1001"),
    ("elpm_r0", &[], "1001 0101 1101 1000"),
    ("eor", &[RD, RR], "0010 01rd dddd rrrr"),
    ("fmul", &[RD, RR], "0000 0011 0ddd 1rrr"),
    ("fmuls", &[RD, RR], "0000 0011 1ddd 0rrr"),
    ("fmulsu", &[RD, RR], "0000 0011 1ddd 1rrr"),
    ("icall", &[], "1001 0101 0000 1001"),
    ("ijmp", &[], "1001 0100 0000 1001"),
    ("in_", &[RD, A], "1011 0AAd dddd AAAA"),
    ("inc", &[RD], "1001 010d dddd 0011"),
    ("jmp", &[ABSOLUTE_OFFSET_DOUBLES], "1001 010k kkkk 110k kkkk kkkk kkkk kkkk"),
    ("lac", &[Arg::ImplicitZ, RD], "1001 001d dddd 0110"),
    ("las", &[Arg::ImplicitZ, RD], "1001 001d dddd 0101"),
    ("lat", &[Arg::ImplicitZ, RD], "1001 001d dddd 0111"),
    ("ldi", &[RD,K], "1110 KKKK dddd KKKK"),
    ("lds_16", &[RD,ABSOLUTE_OFFSET], "1001 000d dddd 0000 kkkk kkkk kkkk kkkk"),
    ("lds_7", &[RD,ABSOLUTE_OFFSET], "1010 0kkk dddd kkkk"),
    ("lpm_r0", &[], "1001 0101 1100 1000"),
    ("lsl", &[RD], "0000 11dd dddd dddd"),
    ("lsr", &[RD], "1001 010d dddd 0110"),
    ("mov", &[RD, RR], "0010 11rd dddd rrrr"),
    ("movw", &[RD_PAIR,RR_PAIR], "0000 0001 dddd rrrr"),
    ("mul", &[RD, RR], "1001 11rd dddd rrrr"),
    ("muls", &[RD, RR], "0000 0010 dddd rrrr"),
    ("mulsu", &[RD, RR], "0000 0011 0ddd 0rrr"),
    ("neg", &[RD], "1001 010d dddd 0001"),
    ("nop", &[], "0000 0000 0000 0000"),
    ("or", &[RD, RR], "0010 10rd dddd rrrr"),
    ("ori", &[RD, K], "0110 KKKK dddd KKKK"),
    ("out", &[A, RR], "1011 1AAr rrrr AAAA"),
    ("pop", &[RD], "1001 000d dddd 1111"),
    ("push", &[RR], "1001 001r rrrr 1111"),
    ("rcall", &[OFFSET], "1101 kkkk kkkk kkkk"),
    ("ret", &[], "1001 0101 0000 1000"),
    ("reti", &[], "1001 0101 0001 1000"),
    ("rjmp", &[OFFSET], "1100 kkkk kkkk kkkk"),
    ("rol", &[RD], "0001 11dd dddd dddd"),
    ("ror", &[RD], "1001 010d dddd 0111"),
    ("sbc", &[RD, RR], "0000 10rd dddd rrrr"),
    ("sbci", &[RD, K], "0100 KKKK dddd KKKK"),
    ("sbi", &[A, B], "1001 1010 AAAA Abbb"),
    ("sbic", &[A, B], "1001 1001 AAAA Abbb"),
    ("sbis", &[A, B], "1001 1011 AAAA Abbb"),
    ("sbiw", &[RD_PAIR, K], "1001 0111 KKdd KKKK"),
    ("sbr", &[RD, K], "0110 KKKK dddd KKKK"),
    ("sbrc", &[RR, B], "1111 110r rrrr 0bbb"),
    ("sbrs", &[RR, B], "1111 111r rrrr 0bbb"),
    ("sec", &[], "1001 0100 0000 1000"),
    ("seh", &[], "1001 0100 0101 1000"),
    ("sei", &[], "1001 0100 0111 1000"),
    ("sen", &[], "1001 0100 0010 1000"),
    ("ser", &[RD], "1110 1111 dddd 1111"),
    ("ses", &[], "1001 0100 0100 1000"),
    ("set", &[], "1001 0100 0110 1000"),
    ("sev", &[], "1001 0100 0011 1000"),
    ("sez", &[], "1001 0100 0001 1000"),
    ("sleep", &[], "1001 0101 1000 1000"),
    ("spm", &[], "1001 0101 1110 1000"),
    ("spm_z_plus", &[], "1001 0101 1111 1000"),
    ("sts", &[ABSOLUTE_OFFSET, RD], "1001 001d dddd 0000 kkkk kkkk kkkk kkkk"),
    ("sub", &[RD, RR], "0001 10rd dddd rrrr"),
    ("subi", &[RD, K], "0101 KKKK dddd KKKK"),
    ("swap", &[RD], "1001 010d dddd 0010"),
    ("tst", &[RD], "0010 00dd dddd dddd"),
    ("wdr", &[], "1001 0101 1010 1000"),
    ("xch", &[Arg::ImplicitZ, RD], "1001 001d dddd 0100"),
];

fn main() {
    let mut lines: Vec<String> = Vec::new();
    
    lines.push("impl Assembler {".to_string());
    
    for &(name, args, template) in INSTRUCTIONS.iter() {
        let arg_strs: Vec<String> = args.iter().map(
            |arg| format!("{}: {}", arg.name(), arg.type_str())
        ).collect();
    
        let arg_intos: Vec<_> = args.iter().map(
            |arg| format!("({}.into(), b'{}')", arg.name(), arg.format_char())
        ).collect();
    
        lines.push(format!("    pub fn {}(&mut self, {}) {{", name, arg_strs.join(", ")));
        for arg in args.iter() {
            match *arg {
                Arg::ImplicitZ => {
                    lines.push("        assert!(z == Z, \"Z (R30,30) is the only accepted argument\");".to_string());
                }
                Arg::RelativeOffset(_) => {
                    lines.push(format!("        let {} = self.resolve_relative_offset({});", arg.name(), arg.name()));
                }
                Arg::AbsoluteOffset(_) => {
                    lines.push(format!("        let {} = self.resolve_absolute_offset({});", arg.name(), arg.name()));
                }
                Arg::AbsoluteOffsetDoubles(_) => {
                    lines.push(format!("        let {} = self.resolve_absolute_offset_doubles({});", arg.name(), arg.name()));
                }
                _ => {}
            }
        }
        
        lines.push(format!("        self.encode(&[{}][..], b{:?})", arg_intos.join(", "), template));
        lines.push("    }".to_string());
    }
    
    lines.push("}".to_string());
    lines.push("".to_string());
    
    let text = lines.join("\n");
    
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR environment variable not set");
    let dest_path = Path::new(&out_dir).join("ops.rs");
    
    File::create(dest_path).and_then(|mut f| f.write_all(&text.as_bytes())).expect("writing ops.rs failed");
}