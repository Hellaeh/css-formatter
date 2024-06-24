// WARNING: KEEP IT COPYABLE
#[derive(Clone, Copy, PartialEq)]
pub enum Token<'a> {
	/// Comment token - will not include starting `/*` and ending `*/`
	Comment(&'a [u8]), // Formatter should preserve comments

	Ident(&'a [u8]),
	Function(&'a [u8]),
	// URL(&'a [u8]),
	// TODO: ^
	// BadURL, // Not supported
	AtRule(&'a [u8]),
	Hash(&'a [u8]),
	/// String token - will not include surrounding quotes
	String(&'a [u8]),
	Number(&'a [u8]),

	Delim(u8),
	// Percentage, // Number
	// Dimension, // Number
	/// Whitespace token - any amount of whitespace(`\s*`)
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

impl<'a> Token<'a> {
	pub fn leak(self) -> Token<'static> {
		unsafe { std::mem::transmute(self) }
	}
}
