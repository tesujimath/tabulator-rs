use clap::{Parser, ValueEnum};
use std::io::{self, Read};
use std::process::exit;

use tabulator::{Align, Cell};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Input format
    #[clap(short)]
    format: Format,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Format {
    Json,
    Psv,
    Psvf,
}

fn main() {
    let cli = Cli::parse();
    let mut input = String::new();

    if let Err(e) = io::stdin().read_to_string(&mut input) {
        eprintln!("Error in input: {}", &e);
        exit(1);
    }

    use Format::*;
    match cli.format {
        Json => {
            match Cell::from_json(&input) {
                Ok(cell) => {
                    println!("{}", &cell);
                }
                Err(e) => {
                    eprintln!("JSON decode error: {}", &e);
                    exit(1);
                }
            };
        }

        Psv => {
            let cell = Cell::from_psv(&input, Align::Left, "  ");
            println!("{}", &cell);
        }

        Psvf => {
            match Cell::from_psvf(&input, Align::Left, "  ") {
                Ok(cell) => {
                    println!("{}", &cell);
                }
                Err(e) => {
                    eprintln!("PSVF error: {}", &e);
                    exit(1);
                }
            };
        }
    }
}
