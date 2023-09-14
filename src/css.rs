use formatter::{Error, Formatter};
use parser::Parser;

use self::parser::Tokens;

pub fn format<'a, S: AsRef<str>>(
	input: &'a S,
	output: &'a mut impl std::io::Write,
) -> Result<(), Error<'a>> {
	let bytes = input.as_ref().as_bytes();

	let parser = Parser::new(bytes);

	let tokens = Tokens::new(parser)?;
	let mut formatter = Formatter::new(tokens, output);

	formatter.format()
}

mod formatter;
mod parser;
mod properties;
