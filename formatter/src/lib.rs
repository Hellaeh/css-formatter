#![feature(array_windows)]
#![feature(assert_matches)]
#![feature(iter_map_windows)]
#![feature(once_cell_get_mut)]
#![feature(specialization)]
#![feature(test)]
#![feature(thread_local)]
#![feature(variant_count)]
#![feature(portable_simd)]
#![feature(debug_closure_helpers)]

use crate::css::Error as CSSError;

#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	NoInput,
	EmptyInput,
	CSS(CSSError<'static>),
	IO(std::io::Error),
}

pub fn format<'a, S: AsRef<[u8]> + 'a>(
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

pub(crate) mod css;
mod utils;
