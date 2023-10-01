// WARNING: KEEP IT COPYABLE
#[derive(Clone, Copy, PartialEq)]
pub enum Token<'a> {
	Comment(&'a [u8]), // Formatter should preserve comments

	Ident(&'a [u8]),
	Function(&'a [u8]),
	// URL(&'a [u8]),
	// BadURL, // Not supported
	AtRule(&'a [u8]),
	Hash(&'a [u8]),
	String(&'a [u8]),
	Delim(u8),
	Number(&'a [u8]),
	// Percentage, // Number
	// Dimension, // Number
	Whitespace,
	Colon,
	Semicolon,
	Comma,
	BracketRoundOpen,
	BracketRoundClose,
	BracketSquareOpen,
	BracketSquareClose,
	BracketCurlyOpen,
	BracketCurlyClose,
}
