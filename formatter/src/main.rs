#[cfg(target_endian = "big")]
fn BigEndian() {
	compile_error!("Big Endian is not supported")
}

use std::io::Read;

use hel_css_formatter::format;
use hel_css_formatter::Error;

fn main() -> Result<(), Error> {
	let mut args = std::env::args();

	let mut writer = std::io::BufWriter::new(std::io::stdout());

	for arg in args.by_ref() {
		if arg == "--input" {
			let Some(input) = args.next() else {
				return Err(Error::NoInput);
			};

			format(input, &mut writer)?;

			return Ok(());
		}
	}

	loop {
		let mut input = String::new();

		// TODO: change to `[u8; 8]` stream
		std::io::stdin().read_to_string(&mut input)?;

		let Err(err) = format(input, &mut writer) else {
			continue;
		};

		match err {
			Error::EmptyInput => break,
			Error::CSS(err) => {
				eprintln!("{err:?}");
				panic!();
			}
			Error::IO(_) => todo!(),
			_ => unreachable!(),
		};
	}

	Ok(())
}
