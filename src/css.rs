use formatter::{Error, Formatter};
use parser::Parser;

pub fn format<'a, S: AsRef<str>>(
	input: &'a S,
	output: &'a mut impl std::io::Write,
) -> Result<(), Error<'a>> {
	let bytes = input.as_ref().as_bytes();

	let mut parser = Parser::new(bytes);
	let mut formatter = Formatter::new();

	formatter.format(&mut parser, output)
}

mod formatter;
mod parser;
mod properties;
