use clap::Parser;

use rand::Rng;

use ca1d::{automate, Border, CAEvalType, Cell, Lattice, Output, CA, CELL0, from_char};

#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "www.github.com/pmmccorm/ca1d")]
struct Opts {
    /// number of symbols (1, 36]
    radix: u32,

    /// neighbor size, centered (must be odd)
    nabor_size: u32,

    /// Wolfram style rule number [0, radix^radix^neighbor_size)
    rule_number: CAEvalType,

    /// initial configuration string in base 36, eg "01f" -> 0, 1, 15
    start_config: String,

    /// neighbor mask NNN.. with N being 0|1, and count of N matching nabor_size
    /// By default all neighbors are evaluated.
    #[clap(short, long, default_value("0"))]
    nabor_mask: String,

    /// level of verbosity
    #[clap(short, long, default_value("0"))]
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

        if self.radix < 2 || self.radix > 36 {
            eprintln!("don't understand CA with {} states", self.radix);
            return false;
        }

        if self.nabor_size % 2 == 0 {
            eprintln!("neighborhood must be an odd number");
            return false;
        }

        if self.nabor_mask != "0" && self.nabor_mask.len() != self.nabor_size as usize {
            eprintln!("neighborhood mask not equal to length");
            return false;
        }

        if ! self.nabor_mask.chars().all(|c| c == '0' || c == '1') {
            eprintln!("neighborhood mask can only contain 0s and 1s");
            return false;
        }

        // we don't validate if rule is too larger here
        // just silently use lower/needed bits

        // @|[0..radix]
        // TODO
        if self.start_config == "@" {
        } else {
            for c in self.start_config.chars() {
                if from_char(c) >= self.radix as u8 {
                    eprintln!("invalid character in given config: {}", c);
                    return false;
                }
            }
        }

        true
    }

    fn hite(&self) -> usize {
        if self.output == Output::UnicodeAnsi {
            return 2 * term_hite(self.to);
        }
        term_hite(self.to)
    }

    fn width(&self) -> usize {
        term_width(self.width)
    }

    // @ means all random config
    // TODO: aborts here if input str has chars not in radix
    fn config(&self) -> Lattice {
        let width = self.width();
        let mut config = Lattice::with_capacity(width);

        if self.start_config == "@" {
            let mut rng = rand::thread_rng();
            for _ in 0..width {
                config.push(rng.gen_range(CELL0..self.radix as Cell));
            }
        } else {
            // normal fill logic
            let padding = (width - self.start_config.len()) / 2;
            let mut lpad = vec![CELL0; padding];
            let mut rpad = vec![CELL0; padding];

            config.append(&mut lpad);

            for c in self.start_config.chars() {
                let d = c.to_digit(self.radix).unwrap();
                config.push(d as Cell);
            }

            config.append(&mut rpad);
        }

        config
    }

    // versus implenenting From trait
    fn to_ca(&self) -> CA {
        CA::new(
            self.nabor_size,
            self.radix,
            self.rule_number.clone(),
            self.border,
        )
    }
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

fn cmd_line(opts: Opts) -> String {
    format!("ca1d {} {} {} {}", opts.radix, opts.nabor_size, opts.rule_number, opts.start_config)
}

pub fn main() {
    let opts: Opts = Opts::parse();

    if !opts.validate_opts() {
        println!("invalid options");
        return;
    }

    let ca = opts.to_ca();

    let (per_s, final_config) = automate(opts.output,
                                         opts.from,
                                         opts.hite(),
                                         &ca,
                                         &opts.config());

    if opts.verbose > 0 {
        eprintln!("\n{} /s", per_s);
        eprintln!("{}", cmd_line(opts));
        eprintln!("{}", CA::print_config(final_config)
        );
    }
}
