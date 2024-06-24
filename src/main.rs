#![feature(array_windows)]
#![feature(assert_matches)]
#![feature(byte_slice_trim_ascii)]
#![feature(iter_map_windows)]
#![feature(once_cell_get_mut)]
#![feature(specialization)]
#![feature(thread_local)]
#![feature(variant_count)]

use crate::css::Error as CSSError;
use std::io::Read;

#[allow(clippy::upper_case_acronyms)]
enum Error {
	NoInput,
	EmptyInput,
	CSS(CSSError<'static>),
	IO(std::io::Error),
}

impl std::process::Termination for Error {
	fn report(self) -> std::process::ExitCode {
		std::process::ExitCode::from(1)
	}
}

impl<'a> From<CSSError<'a>> for Error {
	fn from(value: CSSError<'a>) -> Self {
		match value {
			CSSError::IO(err) => Self::IO(err),
			err => Self::CSS(err.leak()),
		}
	}
}

impl From<std::io::Error> for Error {
	fn from(value: std::io::Error) -> Self {
		Self::IO(value)
	}
}

impl std::fmt::Debug for Error {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Error::NoInput => f.write_str("No input argument been provided after \"--input\" flag!"),
			Error::EmptyInput => f.write_str("Input is empty"),
			Error::CSS(err) => write!(f, "{err:?}"),
			Error::IO(err) => write!(f, "{err:?}"),
		}
	}
}

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

		if let Err(Error::EmptyInput) = format(input, &mut writer) {
			break;
		}
	}

	Ok(())
}

fn format<'a, S: AsRef<[u8]> + 'a>(
	input: S,
	output: &'a mut impl std::io::Write,
) -> Result<(), Error> {
	let input = input.as_ref();

	if input.is_empty() {
		return Err(Error::EmptyInput);
	};

	css::format(input, output)?;

	Ok(())
}

mod consts;
mod css;
mod utils;
