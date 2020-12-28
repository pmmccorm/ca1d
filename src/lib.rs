use std::collections::BTreeMap;
use std::io;
use std::io::{BufWriter, Write};
use std::str::FromStr;
use std::time::Instant;

use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

use num_bigint::BigUint;
use num_traits::cast::ToPrimitive;

// A cell can be in [0,36], limited by from_digit and string input
// A lattice is a 1d array of cells
pub type Cell = u8;
pub type Lattice = Vec<Cell>;

pub const CELL0: Cell = 0;

#[derive(Debug, PartialEq, Clone)]
pub enum CAEvalType {
    Rule(BigUint),
    Code(BigUint),
}

impl CAEvalType {
    fn get_radix(s: &str) -> (u32, &str) {
        if s.len() <= 2 {
            return (10, s);
        }

        let (radix, split) = match &s[..2] {
            "0z" => (36, 2),
            "0x" => (16, 2),
            "0o" => (8, 2),
            "0b" => (2, 2),
            _ => (10, 0),
        };

        let (_, numportion) = s.split_at(split);

        (radix, numportion)
    }

    pub fn new(input: &str) -> Result<Self, &'static str> {
        let mut code = false;

        let (_, s) = if input.starts_with("rule=") {
            input.split_at(5)
        } else if input.starts_with("code=") {
            code = true;
            input.split_at(5)
        } else {
            input.split_at(0)
        };

        let (radix, numportion) = CAEvalType::get_radix(s);

        if numportion == "@" {
        }

        let bn = BigUint::parse_bytes(numportion.as_bytes(), radix);
        match bn {
            None => Err("Failed to parse given rule"),
            Some(n) => {
                if code {
                    Ok(CAEvalType::Code(n))
                } else {
                    Ok(CAEvalType::Rule(n))
                }
            }
        }
    }

    fn to_bignum(&self) -> &BigUint {
        match self {
            CAEvalType::Code(n) => n,
            CAEvalType::Rule(n) => n,
        }
    }
}

impl FromStr for CAEvalType {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<CAEvalType, Self::Err> {
        CAEvalType::new(&input.to_string())
    }
}

struct CAEval {
    eval_type: CAEvalType,
    rule_hash: BTreeMap<usize, Cell>,
    radix: u32,
}

impl CAEval {
    fn new(eval_type: CAEvalType, radix: u32) -> CAEval {
        CAEval {
            eval_type: eval_type.clone(),
            rule_hash: Self::rule_map(eval_type.to_bignum(), radix),
            radix,
        }
    }

    fn eval_idx(&self, idx: usize) -> Cell {
        match self.rule_hash.get(&idx) {
            Some(c) => *c,
            None => CELL0,
        }
    }

    fn eval(&self, naborhood: &Lattice) -> Cell {
        let idx = match self.eval_type {
            CAEvalType::Rule(_) => self.idx_rule(naborhood),
            CAEvalType::Code(_) => self.idx_code(naborhood),
        };

        self.eval_idx(idx)
    }

    // lowest index == lowest bit
    // for rule evaluation
    fn idx_rule(&self, naborhood: &Lattice) -> usize {
        let mut s = String::new();
        for i in naborhood {
            let d = from_digit(i as &Cell);
            s.push(d);
        }

        usize::from_str_radix(&s, self.radix).unwrap()
    }

    // sum-avg / sum mod code function
    fn idx_code(&self, naborhood: &Lattice) -> usize {
        let mut sum: usize = CELL0 as usize;

        for i in naborhood {
            sum += *i as usize;
        }

        sum
    }

    fn rule_map(x: &BigUint, radix: u32) -> BTreeMap<usize, Cell> {
        let mut x: BigUint = x.clone();
        let mut result = BTreeMap::new();
        let mut idx: usize = 0;

        loop {
            let m = x.clone() % radix;
            x /= radix;

            let v: Cell = m.to_u8().unwrap();
            if v != CELL0 {
                result.insert(idx, v);
            }

            idx += 1;

            if x == BigUint::from_slice(&[0]) {
                break;
            }
        }

        //println!("rulemap: {:?}", result);
        result
    }
}

// possibly an optimized from_digit()
pub fn from_digit(c: &Cell) -> char {
    assert!(*c <= 36);

    [
        '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h',
        'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z',
    ][*c as usize]
}

pub fn from_char(c: char) -> Cell {
    u8::from_str_radix(&String::from(c), 36).unwrap()
}

trait CAWriter {
    fn new(radix: u32, width: usize, hite: usize) -> Self
    where
        Self: Sized;

    fn write_line(&mut self, v: &Lattice);
}

struct NullWriter {}
impl CAWriter for NullWriter {
    fn new(_radix: u32, _width: usize, _hite: usize) -> Self {
        Self {}
    }
    fn write_line(&mut self, _v: &Lattice) {}
}

struct RawWriter {}
impl CAWriter for RawWriter {
    fn new(_radix: u32, _width: usize, _hite: usize) -> Self {
        Self {}
    }

    #[allow(unused_must_use)]
    fn write_line(&mut self, v: &Lattice) {
        io::stdout().write_all(v);
    }
}

struct CellWriter {
    sbuf: String,
}
impl CAWriter for CellWriter {
    fn new(_radix: u32, width: usize, _hite: usize) -> Self {
        Self {
            sbuf: String::with_capacity(width),
        }
    }

    fn write_line(&mut self, v: &Lattice) {
        for i in v {
            let c = from_digit(i);
            self.sbuf.push(c);
        }
        println!("{}", self.sbuf);
        self.sbuf.clear();
    }
}

struct AsciiWriter {
    symbols: [char; 5],
    radix: u32,
    sbuf: String,
}
impl CAWriter for AsciiWriter {
    fn new(radix: u32, width: usize, _hite: usize) -> Self {
        assert!(radix <= 4);

        Self {
            symbols: [' ', '-', '=', '#', '@'],
            sbuf: String::with_capacity(width),
            radix,
        }
    }

    fn write_line(&mut self, v: &Lattice) {
        for i in v {
            let idx: usize = idx_select(*i, self.radix, self.symbols.len());
            self.sbuf.push(self.symbols[idx]);
        }
        println!("{}", self.sbuf);
        self.sbuf.clear();
    }
}

struct UnicodeWriter {
    ascii_writer: AsciiWriter,
}
impl CAWriter for UnicodeWriter {
    fn new(radix: u32, width: usize, _hite: usize) -> Self {
        Self {
            ascii_writer: AsciiWriter {
                symbols: [' ', '░', '▒', '▓', '█'],
                sbuf: String::with_capacity(width),
                radix,
            },
        }
    }

    fn write_line(&mut self, v: &Lattice) {
        self.ascii_writer.write_line(v)
    }
}

struct AnsiGreyWriter {
    bufwtr: termcolor::BufferWriter,
    greys: Vec<Color>,
    radix: u32,
}
impl CAWriter for AnsiGreyWriter {
    fn new(radix: u32, _width: usize, _hite: usize) -> Self {
        let mut greyscale = Vec::new();

        for c in 232..=255 {
            greyscale.push(Color::Ansi256(c));
        }

        Self {
            bufwtr: BufferWriter::stdout(ColorChoice::Always),
            greys: greyscale,
            radix,
        }
    }

    #[allow(unused_must_use)]
    fn write_line(&mut self, v: &Lattice) {
        let mut buffer = self.bufwtr.buffer();

        for i in v {
            let idx: usize = idx_select(*i, self.radix, self.greys.len());
            buffer.set_color(ColorSpec::new().set_bg(Some(self.greys[idx])));
            write!(&mut buffer, " ");
        }

        buffer.reset();
        writeln!(&mut buffer);

        self.bufwtr.print(&buffer);
    }
}

struct UnicodeAnsiWriter {
    bufwtr: termcolor::BufferWriter,
    colors: Vec<Color>,
    radix: u32,
    config: Option<Lattice>,
}
impl CAWriter for UnicodeAnsiWriter {
    fn new(radix: u32, _width: usize, _hite: usize) -> Self {
        let mut colors = Vec::new();

        for c in 0..radix as u8 {
            let (r, g, b) = cell_to_rgb(c, radix);
            colors.push(Color::Rgb(r, g, b));
        }

        Self {
            bufwtr: BufferWriter::stdout(ColorChoice::Always),
            colors,
            radix,
            config: None,
        }
    }

    #[allow(unused_must_use)]
    fn write_line(&mut self, v: &Lattice) {
        let mut buffer = self.bufwtr.buffer();
        let top: Lattice;

        match &self.config {
            Some(t) => top = t.to_vec(),
            None => {
                self.config = Some(v.to_vec());
                return;
            }
        }

        for (i, _) in top.iter().enumerate() {
            let idx_top: usize = idx_select(top[i], self.radix, self.colors.len());
            let idx_bot: usize = idx_select(v[i], self.radix, self.colors.len());
            buffer.set_color(
                ColorSpec::new()
                    .set_fg(Some(self.colors[idx_top]))
                    .set_bg(Some(self.colors[idx_bot])),
            );

            write!(&mut buffer, "▀");
        }

        buffer.reset();
        writeln!(&mut buffer);

        self.bufwtr.print(&buffer);
        self.config = None;
    }
}

impl Drop for UnicodeAnsiWriter {
    // we could have a previously written line cached, flush it here
    fn drop(&mut self) {
        if let Some(v) = &self.config {
            let pad: Lattice = [CELL0].repeat(v.len());
            self.write_line(&pad);
        }
    }
}

struct PNGWriter {
    fd: png::StreamWriter<'static, std::io::BufWriter<std::io::Stdout>>,
    radix: u32,
}
impl CAWriter for PNGWriter {
    fn new(radix: u32, width: usize, hite: usize) -> Self {
        let w: std::io::BufWriter<std::io::Stdout> = BufWriter::new(std::io::stdout());
        let mut encoder = png::Encoder::new(w, width as u32, hite as u32);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let writer = encoder.write_header().unwrap();

        Self {
            fd: writer.into_stream_writer(),
            radix,
        }
    }

    // TODO: optimize
    #[allow(unused_must_use)]
    fn write_line(&mut self, v: &Lattice) {
        let mut line = Vec::new();
        for i in v {
            let (r, g, b) = cell_to_rgb(*i, self.radix);
            line.push(r);
            line.push(g);
            line.push(b);
        }
        self.fd.write(&line);
    }
}

fn to_base_triple(c: Cell, radix: u32) -> Vec<f32> {
    let mut x: u32 = c.into();
    let mut result = Vec::new();

    loop {
        let m = x % radix;
        x /= radix;

        result.push(m as f32 / (radix - 1) as f32);

        if x == 0 {
            break;
        }
    }

    while result.len() <= 3 {
        result.push(0.0);
    }

    result
}

// covers up to 64 symbols..base 7 will bring to 343
// convert the given cell [0,255] into a differently based digit that fits into three
// symbols, ie 10 -> 130 (10 in base 3)
// then scale the R,G,B values by this triple.
// makes for maximally distant, but somewhat ugly colors
fn cell_to_rgb(c: Cell, radix: u32) -> (u8, u8, u8) {
    assert!(radix <= 64);
    let base = if radix < 8 {
        2
    } else if radix < 27 {
        3
    } else {
        4
    };

    let t = to_base_triple(c, base);

    (
        (t[0] * 255.0) as u8,
        (t[1] * 255.0) as u8,
        (t[2] * 255.0) as u8,
    )
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Output {
    Null,
    Ascii,
    AnsiGrey,
    Unicode,
    UnicodeAnsi,
    Cell,
    PNG,
    Raw,
}

fn get_printer(o: Output, radix: u32, width: usize, hite: usize) -> Box<dyn CAWriter> {
    match o {
        Output::Null => Box::new(NullWriter::new(radix, width, hite)),
        Output::Cell => Box::new(CellWriter::new(radix, width, hite)),
        Output::Ascii => Box::new(AsciiWriter::new(radix, width, hite)),
        Output::Unicode => Box::new(UnicodeWriter::new(radix, width, hite)),
        Output::AnsiGrey => Box::new(AnsiGreyWriter::new(radix, width, hite)),
        Output::UnicodeAnsi => Box::new(UnicodeAnsiWriter::new(radix, width, hite)),
        Output::PNG => Box::new(PNGWriter::new(radix, width, hite)),
        Output::Raw => Box::new(RawWriter::new(radix, width, hite)),
    }
}

// TODO: matching to the above correct fn is stored in CAPrinter struct.
// I want it to be in the enum..
impl FromStr for Output {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Null" => Ok(Output::Null),
            "Ascii" => Ok(Output::Ascii),
            "AnsiGrey" => Ok(Output::AnsiGrey),
            "Unicode" => Ok(Output::Unicode),
            "UnicodeAnsi" => Ok(Output::UnicodeAnsi),
            "Cell" => Ok(Output::Cell),
            "PNG" => Ok(Output::PNG),
            "Raw" => Ok(Output::Raw),
            _ => Err("invalid output type"),
        }
    }
}

// TODO: specify the fixed border symbol(s?)
#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Border {
    Ring,
    Fixed,
}

impl FromStr for Border {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "ring" => Ok(Border::Ring),
            "fixed" => Ok(Border::Fixed),
            _ => Err("invalid border style"),
        }
    }
}

pub struct CA {
    nabor_size: u32,
    rule_order: u32,
    border: Border,
    rule: CAEval,
}

impl CA {
    pub fn print_config(l: Lattice) -> String {
        let config: String = l.iter().map(from_digit).collect();
        let config = config.trim_start_matches(from_digit(&CELL0));
        let config = config.trim_end_matches(from_digit(&CELL0));

        config.to_string()
    }

    // does no checking of inputs, watch out
    // width == config.len()
    pub fn new(
        nabor_size: u32,
        rule_order: u32,
        rule_number: CAEvalType,
        border: Border,
    ) -> CA {
        CA {
            nabor_size,
            rule_order,
            border,
            rule: CAEval::new(rule_number, rule_order),
        }
    }

    // fixed border of lowest symbol
    fn eval_fixed(&self, config: &Lattice) -> Vec<u8> {
        let mut next: Vec<u8> = Vec::with_capacity(config.len());
        let nabor_size = self.nabor_size as usize;
        let nabor_side = (nabor_size - 1) / 2;
        let mut nabors: Vec<u8> = Vec::with_capacity(nabor_size);

        for idx in 0..config.len() {
            let idx: i32 = idx as i32;

            for i in (idx - nabor_side as i32)..(idx + nabor_side as i32) + 1 {
                if i < 0 || i >= config.len() as i32 {
                    nabors.push(0);
                } else {
                    nabors.push(config[i as usize]);
                }
            }

            next.push(self.rule.eval(&nabors));
            nabors.clear();
        }

        next
    }

    fn eval_ring(&self, config: &Lattice) -> Lattice {
        let mut next: Vec<u8> = Vec::with_capacity(config.len());
        let nabor_size = self.nabor_size as usize;
        let nabor_side = (nabor_size - 1) / 2;
        let mut nabors: Lattice = Vec::with_capacity(nabor_size);

        for idx in 0..config.len() {
            let idx: i32 = idx as i32;

            for i in (idx - nabor_side as i32)..(idx + nabor_side as i32) + 1 {
                nabors.push(config[idx_mod(i, config.len())]);
            }

            next.push(self.rule.eval(&nabors));
            nabors.clear();
        }

        next
    }

    pub fn gtf(&self, config: &Lattice) -> Lattice {
        match self.border {
            Border::Ring => self.eval_ring(config),
            Border::Fixed => self.eval_fixed(config),
        }
    }
}

pub struct CAPrinter<'a> {
    output: Box<dyn CAWriter>,
    ca: &'a CA,
}

impl CAPrinter<'_> {
    pub fn new(output: Output, ca: & CA, width: usize, hite: usize) -> CAPrinter {
        CAPrinter {
            output: get_printer(output, ca.rule_order, width, hite),
            ca,
        }
    }

    // returns cells per second
    fn eval(&mut self, from: usize, count: usize, config: & Lattice) -> (f64, Lattice) {
        let mut config = config.clone();
        let start = Instant::now();

        for _ in 0..from {
            config = self.ca.gtf(&config);
        }

        for _ in 0..count {
            self.output.write_line(&config);
            config = self.ca.gtf(&config);
        }

        (
            ((from + count) * config.len()) as f64 / start.elapsed().as_secs_f64(),
            config,
        )
    }
}

// given a symbol number in [0,sym_count) , sym_count, and array_len >= sym_count
// return and index in [0,array_len) that is linearly "spaced" equally over the array
fn idx_select(sym_num: u8, sym_count: u32, array_len: usize) -> usize {
    let sym_num: usize = sym_num as usize;
    let sym_count: usize = sym_count as usize;

    let space = array_len / (sym_count - 1);
    let idx = usize::min(sym_num * space, array_len - 1);

    assert!(sym_count > 1);
    assert!(array_len > 1);
    assert!(sym_num < sym_count);
    assert!(sym_count <= array_len);

    idx
}

// given a singed integer and an array length, treat the integer as
// an index and "roll" it over the array
// ie modulo with defined behavior for negative numbers
// TODO: our lattice size is limited to i32 (2^31)
fn idx_mod(idx: i32, array_len: usize) -> usize {
    let array_len: i32 = array_len as i32;

    if idx >= array_len {
        (idx % array_len) as usize
    } else if idx < 0 {
        (array_len + (idx % array_len)) as usize
    } else {
        idx as usize
    }
}

pub fn automate(output: Output, from: usize, to: usize, ca: &CA, start_config: & Lattice) -> (f64, Lattice) {
    let width = start_config.len();
    let mut output = CAPrinter::new(output, ca, width, to);
    output.eval(from, to, start_config)
}
