use std::str::FromStr;
use std::char::from_digit;
use std::time::{Instant};
use std::fmt;

//use std::ops::{Index};
//use std::iter::{Iterator, ExactSizeIterator};

//use std::convert::TryInto;
//use std::num::ParseIntError;
//use std::collections::HashMap;

use clap::Clap;
use term_size;

use std::io::{Write};
use termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor};

// A cell can be in [0,36], limited by from_digit and string input
// A lattice is a 1d array of cells
type Cell = u8;
type Lattice = Vec<Cell>;

const CELL0 : Cell = 0;

#[derive(Debug)]
struct Rule {
	r : Vec<Cell>,
	rule_order : u32,
}

impl fmt::Display for Rule {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "rule")
	}
}

impl Rule {
	fn new(mut from_str : String, rule_order : u32, nabor_size : u32) -> Rule {
			Self { r: Self::itov(parse_u128(&mut from_str), rule_order, nabor_size),
				rule_order: rule_order,
			}
	}

	fn rule_size_or_max(rule_order : u32, nabor_size : u32) -> usize {
		let (rs, overflow) = (rule_order as usize).overflowing_pow(nabor_size);
		if overflow {
			usize::MAX
		} else {
			rs
		}
	}

	// lowest index == lowest bit
	fn at_vec(&self, nhood: & Lattice) -> Cell {
		let mut s = String::new();
		for i in nhood {
			let d = from_digit(*i as u32, self.rule_order).unwrap();
			s.push(d);
		}

		let subrule : usize = usize::from_str_radix(&s, self.rule_order).unwrap();

		self.at(subrule)
	}

	fn at(&self, idx : usize) -> Cell {
		if idx >= self.r.len() {
			CELL0
		} else {
			self.r[idx]
		}
	}

	// TODO: suport base 1?
	// TODO: radix is not checked, but is really limited to [2,36]
	// NOTE: returns vector of u8 [0,35] for given rule, eg:
	// rule 1, space/radix 2: [1]
	// rule 6, space/radix 3: [0, 2, 0, ... 0 ]
	// lsb is lowest index
	fn itov(mut x: u128, radix: u32, nabor_size : u32) -> Vec<Cell> {
		let mut result = vec![];
		let rule_size = Self::rule_size_or_max(radix, nabor_size);

		loop {
			let m = x % (radix as u128);
			x = x / (radix as u128);

			result.push(m as u8);
			if x == 0 {
				break;
			}

			if result.len() >= rule_size {
				break;
			}
		}

		result.into_iter().collect()
	}
}

fn null_writer(_order : u32, _v : & Vec<u8>) -> () {
}

fn cell_writer(order : u32, v : & Vec<u8>) -> () {
	let mut sline = String::from("");
	for i in v {
		let c = from_digit(*i as u32, order).unwrap();
		sline.push(c);
	}
	println!("{}", sline);
}

fn ascii_writer(order : u32, v : & Vec<u8>) -> () {
	let symbols = [' ', '=', '#'];
	let mut sline = String::from("");

	for i in v {
		let idx : usize = idx_select(*i, order, symbols.len());
		sline.push(symbols[idx]);
	}
	println!("{}", sline);
}

fn unicode_writer(order : u32, v : & Vec<u8>) -> () {
	let symbols = [ ' ', '░', '▒', '▓', '█' ];
	let mut sline = String::from("");

	for i in v {
		let idx : usize = idx_select(*i, order, symbols.len());
		sline.push(symbols[idx]);
	}
	println!("{}", sline);
}

#[allow(unused_must_use)]
fn ansi_writer(order : u32, v : & Vec<u8>) -> () {
	let greyscale = [ Color::Ansi256(232), Color::Ansi256(233), Color::Ansi256(234),
			  Color::Ansi256(235), Color::Ansi256(236), Color::Ansi256(237),
			  Color::Ansi256(237), Color::Ansi256(238), Color::Ansi256(239),
			  Color::Ansi256(240), Color::Ansi256(241), Color::Ansi256(242),
			  Color::Ansi256(243), Color::Ansi256(244), Color::Ansi256(245),
			  Color::Ansi256(246), Color::Ansi256(247), Color::Ansi256(248),
			  Color::Ansi256(249), Color::Ansi256(250), Color::Ansi256(251),
			  Color::Ansi256(252), Color::Ansi256(253), Color::Ansi256(254),
			  Color::Ansi256(255)
	];
	let bufwtr = BufferWriter::stdout(ColorChoice::Always);
	let mut buffer = bufwtr.buffer();

	for i in v {
		let idx : usize = idx_select(*i, order, greyscale.len());
		buffer.set_color(ColorSpec::new().set_fg(Some(greyscale[idx])));
		write!(&mut buffer, "█");
	}

	buffer.reset();
	write!(&mut buffer, "\n");

	bufwtr.print(&buffer);
}

#[allow(unused_must_use)]
fn unicode2_writer(order : u32, top : & Vec<u8>, bot : & Vec<u8>) -> () {
	let colormap = [ Color::Ansi256(0), Color::Ansi256(1), Color::Ansi256(2),
			 Color::Ansi256(3), Color::Ansi256(4), Color::Ansi256(5),
			 Color::Ansi256(6), Color::Ansi256(7), Color::Ansi256(8),
			 Color::Ansi256(9), Color::Ansi256(10), Color::Ansi256(11),
	];
	let bufwtr = BufferWriter::stdout(ColorChoice::Always);
	let mut buffer = bufwtr.buffer();

	assert_eq!(top.len(), bot.len());

	for (i, _) in top.iter().enumerate() {
		let idx_top : usize = idx_select(top[i], order, colormap.len());
		let idx_bot : usize = idx_select(bot[i], order, colormap.len());
		buffer.set_color(
			ColorSpec::new().set_fg(Some(colormap[idx_top])).set_bg(Some(colormap[idx_bot])));


		write!(&mut buffer, "▀");
	}

	buffer.reset();
	write!(&mut buffer, "\n");

	bufwtr.print(&buffer);
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum Output {
    Null,
    Ascii,
    Ansi,
    Unicode,
    Unicode2,
    Cell,
}

type UV = fn(u32, & Vec<u8>) -> ();
type UVV = fn(u32, & Vec<u8>, & Vec<u8>) -> ();
enum Printer {
	ByOne(UV),
	ByTwo(UVV),
}

fn get_printer(o : Output) -> Printer {
	match o {
		Output::Null => Printer::ByOne(null_writer),
		Output::Ascii => Printer::ByOne(ascii_writer),
		Output::Ansi => Printer::ByOne(ansi_writer),
		Output::Unicode => Printer::ByOne(unicode_writer),
		Output::Unicode2 => Printer::ByTwo(unicode2_writer),
		Output::Cell => Printer::ByOne(cell_writer),
	}
}

// TODO: matching to the above correct fn is stored in CAPrinter struct.
// I want it to be in the enum..
impl FromStr for Output {
    type Err = &'static str;

     fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "Null"     => Ok(Output::Null),
            "Ascii"     => Ok(Output::Ascii),
            "Ansi"     => Ok(Output::Ansi),
            "Unicode"   => Ok(Output::Unicode),
            "Unicode2"   => Ok(Output::Unicode2),
            "Cell"   => Ok(Output::Cell),
            _           => Err("invalid output type"),
        }
    }
}

// TODO: specify the fixed border symbol(s?)
#[derive(Debug, PartialEq, Copy, Clone)]
enum Border {
	Ring,
	Fixed,
}

impl FromStr for Border {
    type Err = &'static str;

     fn from_str(input: &str) -> Result<Self, Self::Err> {
	match input {
		"ring"	=> Ok(Border::Ring),
		"fixed"	=> Ok(Border::Fixed),
		_	=> Err("invalid border style"),
	}
    }
}

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "Patrick McCormick <patm.mail@gmail.com>")]
struct Opts {
    /// number of symbols (1, 36]
    rule_order: u32,

    /// neighbor size, centered (must be odd)
    nabor_size: u32,

    /// Wolfram style rule number [0, order^order^neighbor_size)
    rule_number: String,

    /// initial configuration string in base 36, eg "01f" -> 0, 1, 15
    start_config: String,

    /// level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// select output type
    #[clap(short, long, default_value("Unicode2"))]
    output: Output,

    /// border behavior: ring or fixed
    #[clap(short, long, default_value("ring"))]
    border: Border,

    /// width of lattice, length of 0 will pick terminal width
    #[clap(long, default_value("0"))]
    width: usize,

    /// length of automata: N or 0 will choose terminal heigth
    #[clap(long, default_value("0"))]
    hite: usize,

    /// start displaying automation after N steps
    #[clap(long, default_value("0"))]
    from: usize,
}

struct CA {
	config: Lattice,
	nabor_size: u32,
	rule_order: u32,
	border: Border,
	rule : Rule,
}

impl fmt::Display for CA {

	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "ca1d {} {} {} {}",
		       self.rule_order,
		       self.nabor_size,
		       self.rule,
		       "")
	}
}

impl CA {
	fn init_config(config : & String, width : usize) -> Vec<u8> {
		let mut v = Lattice::with_capacity(width);
		let padding = (width - config.len()) / 2;
		let mut lpad = vec![CELL0; padding];
		let mut rpad = vec![CELL0; padding];

		assert!(config.len() <= width);

		v.append(&mut lpad);

		for c in config.chars() {
			let d = c.to_digit(36).unwrap();
			v.push(d as u8);
		}

		v.append(&mut rpad);

		v
	}

	fn from_opts(opts: &Opts) -> CA {
		let width = term_width(opts.width);
		CA {config: CA::init_config(& opts.start_config, width),
			nabor_size: opts.nabor_size,
			rule_order: opts.rule_order,
			border: opts.border,
			rule: Rule::new(opts.rule_number.clone(), opts.rule_order, opts.nabor_size)
		}
	}

	// fixed border of lowest symbol
	fn eval_fixed(&self, config: & Vec<u8>) -> Vec<u8> {
		let mut next : Vec<u8> = Vec::with_capacity(config.len());
		let nabor_size = self.nabor_size as usize;
		let nabor_side = (nabor_size - 1) / 2;

		for idx in 0 .. config.len() {
			let idx : i32 = idx as i32;
			let mut nabors : Vec<u8> = Vec::with_capacity(nabor_size);

			for i in (idx - nabor_side as i32) .. (idx + nabor_side as i32) + 1 {
				if i < 0 || i >= config.len() as i32 {
					nabors.push(0);
				} else {
					nabors.push(config[i as usize]);
				}
			}

			next.push(self.rule.at_vec(&nabors));
		}

		return next;
	}

	fn eval_ring(&self, config: & Vec<u8>) -> Vec<u8> {
		let mut next : Vec<u8> = Vec::with_capacity(config.len());
		let nabor_size = self.nabor_size as usize;
		let nabor_side = (nabor_size - 1) / 2;

		for idx in 0 .. config.len() {
			let idx : i32 = idx as i32;
			let mut nabors : Vec<u8> = Vec::with_capacity(nabor_size);

			for i in (idx - nabor_side as i32) .. (idx + nabor_side as i32) + 1 {
				nabors.push(config[idx_mod(i, config.len())]);
			}

			next.push(self.rule.at_vec(&nabors));
		}

		return next;
	}

	fn gtf(&self, config : & Vec<u8>) -> Vec<u8> {
		match self.border {
			Border::Ring => {
				self.eval_ring(config)
			}
			Border::Fixed => {
				self.eval_fixed(config)
			}
		}
	}
}

struct CAPrinter<'a> {
	ca : &'a CA,
	printerfn : Printer,
	hite: usize,
	from: usize,
}

impl CAPrinter<'_> {
	fn new<'a>(opts: &'a Opts, ca: &'a CA) -> CAPrinter<'a> {
		CAPrinter { ca : ca,
			    printerfn : get_printer(opts.output),
			    from: opts.from,
			    hite: term_hite(opts.hite)
		}
	}

	fn eval(&self) -> f64 {
		self.eval_print(self.from, self.hite)
	}

	// returns gtfs per second
	fn eval_print(&self, from : usize, count : usize) -> f64 {
		let mut config = self.ca.config.clone();
		let start = Instant::now();

		for _ in 0 .. from {
			null_writer(self.ca.rule_order, &config);
			config = self.ca.gtf(&config);
		}

		match self.printerfn {
			Printer::ByOne(f) => {
				for _ in 0 .. count {
					f(self.ca.rule_order, &config);
					config = self.ca.gtf(&config);
				}
			}
			Printer::ByTwo(f) => {
				let mut config2;
				for _ in 0 .. count / 2 {
					config2 = self.ca.gtf(&config.clone());
					f(self.ca.rule_order, &config, &config2);
					config = self.ca.gtf(&config2);
				}
			}
		}

		return (from + count) as f64 / start.elapsed().as_secs() as f64;
	}
}

fn parse_u128(s : &mut String) -> u128 {
	if s.len() <= 2 {
		u128::from_str_radix(s, 10).unwrap()
	} else {

		let (radix, s) = match &s[..2] {
			"0z" => (36, s.split_off(2)),
			"0x" => (16, s.split_off(2)),
			"0o" => (8, s.split_off(2)),
			"0b" => (2, s.split_off(2)),
			_    => (10, s.split_off(0)),
		};

		u128::from_str_radix(s.as_str(), radix).unwrap()
	}
}

// given a symbol number in [0,sym_count) , sym_count, and array_len >= sym_count
// return and index in [0,array_len) that is linearly "spaced" equally over the array
fn idx_select(sym_num : u8, sym_count : u32, array_len : usize) -> usize {
	let sym_num : usize = sym_num as usize;
	let sym_count : usize = sym_count as usize;

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
// ie modulo which defined behavior for negative numbers
// TODO: our lattice size is limited to i32 (2^31)
fn idx_mod(idx: i32, array_len: usize) -> usize {
	let array_len : i32 = array_len as i32;

	if idx >= array_len {
		(idx % array_len) as usize
	} else if idx < 0 {
		(array_len + (idx % array_len)) as usize
	} else {
		idx as usize
	}
}

fn validate_opts(opts: &Opts) -> bool {
	if opts.verbose > 0 {
		println!("{:?}", opts);
	}

	if opts.rule_order < 2 || opts.rule_order > 36 {
		println!("don't understand CA with {} states", opts.rule_order);
		return false;
	}

	if opts.nabor_size % 2 == 0 {
		println!("neighborhood must be an odd number");
		return false;
	}

	// we don't validate if rule is too larger here
	// just silently use lower/needed bits

	// now validate start_config...
	// symbols [0..rule_order)
	for c in opts.start_config.chars() {
		let d = c.to_digit(36).unwrap();
		if d >= opts.rule_order.into() {
			return false;
		}
	}

	return true;
}

fn term_wh() -> (usize, usize) {
	if let Some((w, h)) = term_size::dimensions_stdout() {
		(w, h)
	} else {
		(80, 25)
	}
}

fn term_hite(from_opts: usize) -> usize {
	let (_, h) = term_wh();

	if from_opts == 0 {
		h
	} else {
		from_opts
	}
}

fn term_width(from_opts: usize) -> usize {
	let (w, _) = term_wh();

	if from_opts == 0 {
		w
	} else {
		from_opts
	}
}

fn main() {
	let opts: Opts = Opts::parse();

	if ! validate_opts(&opts) {
		println!("invalid options");
		return ();
	}

	let ca: CA = CA::from_opts(&opts);

	let output = CAPrinter::new(&opts, &ca);
	let per_s = output.eval();

	println!("\n{} /s", per_s);

	if opts.verbose > 0 {
		println!("{}", ca);
	}
}
