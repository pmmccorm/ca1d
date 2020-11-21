extern crate clap;
use crate::clap::Clap;

use rand::Rng;
use term_size;

use ca1d::{automate, Border, CAEvalType, Cell, Lattice, Output, CA, CELL0};

#[derive(Clap, Debug)]
#[clap(version = "1.0", author = "www.github.com/pmmccorm/ca1d")]
struct Opts {
    /// number of symbols (1, 36]
    rule_order: u32,

    /// neighbor size, centered (must be odd)
    nabor_size: u32,

    /// Wolfram style rule number [0, order^order^neighbor_size)
    rule_number: CAEvalType,

    /// initial configuration string in base 36, eg "01f" -> 0, 1, 15
    start_config: String,

    /// level of verbosity
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,

    /// select output type
    #[clap(short, long, default_value("UnicodeAnsi"))]
    output: Output,

    /// border behavior: ring or fixed
    #[clap(short, long, default_value("ring"))]
    border: Border,

    /// width of lattice, length of 0 will pick terminal width
    #[clap(long, default_value("0"))]
    width: usize,

    /// length of automata: N or 0 will choose terminal heigth
    #[clap(long, default_value("0"))]
    to: usize,

    /// start displaying automation after N steps
    #[clap(long, default_value("0"))]
    from: usize,
}

impl Opts {
    fn validate_opts(&self) -> bool {
        if self.verbose > 2 {
            eprintln!("{:?}", self);
        }

        if self.rule_order < 2 || self.rule_order > 36 {
            eprintln!("don't understand CA with {} states", self.rule_order);
            return false;
        }

        if self.nabor_size % 2 == 0 {
            eprintln!("neighborhood must be an odd number");
            return false;
        }

        // we don't validate if rule is too larger here
        // just silently use lower/needed bits

        // now validate start_config...
        // symbols [0..rule_order)
        // TODO: let config_transform handle this..

        return true;
    }

    // versus implenenting From trait
    fn to_ca(&self) -> CA {
        let width = term_width(self.width);
        let start_config = config_transform(self.rule_order, width, &self.start_config);
        CA::new(
            start_config,
            self.nabor_size,
            self.rule_order,
            self.rule_number.clone(),
            self.border,
        )
    }
}

// for now @ means all random config
// in future: _ for a random character? eg 1__2_0
// TODO: aborts here if input str has chars not in radix
fn config_transform(radix: u32, width: usize, s: &String) -> Lattice {
    let mut config = Lattice::with_capacity(width);

    assert!(config.len() <= width);

    if s == "@" {
        let mut rng = rand::thread_rng();
        for _ in 0..width {
            config.push(rng.gen_range(CELL0, radix as Cell));
        }
    } else {
        // normal fill logic
        let padding = (width - s.len()) / 2;
        let mut lpad = vec![CELL0; padding];
        let mut rpad = vec![CELL0; padding];

        config.append(&mut lpad);

        for c in s.chars() {
            let d = c.to_digit(radix).unwrap();
            config.push(d as Cell);
        }

        config.append(&mut rpad);
    }

    config
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

pub fn main() {
    let opts: Opts = Opts::parse();

    if !opts.validate_opts() {
        println!("invalid options");
        return ();
    }

    let ca = opts.to_ca();

    let (per_s, final_config) = automate(opts.output, opts.from, term_hite(opts.to), &ca);

    if opts.verbose > 0 {
        eprintln!("\n{} /s", per_s);

        eprintln!(
            "ca1d {} {} {:?} {}",
            opts.rule_order,
            opts.nabor_size,
            opts.rule_number,
            CA::print_config(final_config)
        );
    }
}
