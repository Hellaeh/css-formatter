#![feature(variant_count)]
#![feature(byte_slice_trim_ascii)]

use std::io::{Read, Write};

fn main() -> std::io::Result<()> {
	let mut args = std::env::args();

	for arg in args.by_ref() {
		if arg == "--input" {
			break;
		}
	}

	let Some(input) = args.next().or_else(|| {
		let mut res = String::new();
		let mut reader = std::io::BufReader::new(std::io::stdin());

		reader
			.read_to_string(&mut res)
			.unwrap_or_else(|e| panic!("Error while reading stdin: {e:?}"));

		Some(res)
	}) else {
		panic!("No input were provided");
	};

	if input.is_empty() {
		panic!("Empty input string");
	}

	let mut output = Vec::with_capacity(input.len());

	if let Err(error) = css::format(&input, &mut output) {
		eprintln!("Error: {error:?}");
		std::process::exit(1);
	};

	std::io::stdout().write_all(&output)
}

pub mod css;
pub mod utils;
