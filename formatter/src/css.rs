pub use formatter::Error;

use formatter::Formatter;
use parser::Parser;

use parser::Cache;

pub fn format<'a>(input: &'a [u8], output: &'a mut impl std::io::Write) -> Result<(), Error<'a>> {
	let parser = Parser::new(input);

	let cache = Cache::new(parser)?;
	let mut formatter = Formatter::new(cache, output);

	formatter.format()
}

mod formatter;
mod parser;
pub(crate) mod properties;
mod tokens;
