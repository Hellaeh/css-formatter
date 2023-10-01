#![feature(byte_slice_trim_ascii)]
#![feature(variant_count)]
#![feature(specialization)]

use std::io::Read;

fn main() -> std::io::Result<()> {
	let mut args = std::env::args();

	let mut writer = std::io::BufWriter::new(std::io::stdout());

	for arg in args.by_ref() {
		if arg == "--input" {
			let Some(input) = args.next() else {
				panic!("No input were provided");
			};

			if input.is_empty() {
				panic!("Empty input string");
			}

			if let Err(error) = css::format(&input, &mut writer) {
				eprintln!("Error: {error:?}");
				std::process::exit(1);
			};

			return Ok(());
		}
	}

	loop {
		let mut input = String::new();

		std::io::stdin().read_to_string(&mut input)?;

		if input.is_empty() {
			break;
		};

		if let Err(error) = css::format(&input, &mut writer) {
			eprintln!("Error: {error:?}");
			std::process::exit(1);
		};
	}

	Ok(())
}

mod consts;
mod css;
mod utils;
