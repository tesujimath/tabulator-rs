use std::io::{self, Read};
use std::process::exit;

use tabulator::Cell;

fn main() {
    let mut cell_json = String::new();

    if let Err(e) = io::stdin().read_to_string(&mut cell_json) {
        eprintln!("Error in input: {}", &e);
        exit(1);
    }

    match Cell::from_json(&cell_json) {
        Ok(cell) => {
            println!("{}", &cell);
        }
        Err(e) => {
            eprintln!("JSON decode error: {}", &e);
            exit(1);
        }
    };
}
