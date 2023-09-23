mod tokens;

use crate::utils::Helper;

pub use self::tokens::Tokens;

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

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	BadComment,
	BadString,
	EOF,
	NonASCII,
}

/// An opinionated parser for opinionated CSS formatter
#[derive(Debug)]
pub struct Parser<'a> {
	buf: &'a [u8],
	// Current position (index)
	pos: usize,
	// For `peek`ing
	cache: Option<Token<'a>>,
}

pub type Result<T> = std::result::Result<T, Error>;

impl<'a> Parser<'a> {
	#[inline(always)]
	pub fn advance(&mut self, amount: usize) {
		self.pos += amount
	}

	#[inline(always)]
	pub fn is_eof(&self) -> bool {
		self.pos() >= self.buf.len()
	}

	#[inline]
	pub fn new(input: &'a [u8]) -> Self {
		Self {
			buf: input,
			pos: 0,
			cache: None,
		}
	}

	#[inline]
	pub fn next(&mut self) -> Result<Token<'a>> {
		if self.cache.is_some() {
			return Ok(self.cache.take().unwrap());
		}

		let bytes = self.buf;

		if self.is_eof() {
			return Err(Error::EOF);
		}

		let cur = unsafe { bytes.get_unchecked(self.pos()) };
		let next = bytes.get(self.pos() + 1);

		if !cur.is_ascii() {
			return Err(Error::NonASCII);
		}

		let token = match cur {
			// A comment or delim token
			b'/' => {
				if next == Some(&b'*') {
					self.parse_comment(bytes)?
				} else {
					self.advance(1);
					Token::Delim(b'/')
				}
			}

			// Whitespace token
			b' ' | b'\t' | b'\n' | b'\r' => {
				self.skip_whitespace(bytes);
				Token::Whitespace
			}

			// A hash or delim token
			b'#' => {
				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | b'_')) {
					self.parse_name(bytes)
				} else {
					self.advance(1);
					Token::Delim(b'#')
				}
			}

			// String token
			quote_type @ (b'\'' | b'"') => {
				// Step over opening quote
				self.advance(1);
				self.parse_string(bytes, quote_type)?
			}

			// Number token
			b'0'..=b'9' => self.parse_number(bytes)?,

			// Number or delim token
			#[allow(unused_must_use)]
			delim @ (b'.' | b'+') => {
				if next.is_digit() {
					self.parse_number(bytes)?
				} else {
					self.advance(1);
					Token::Delim(*delim)
				}
			}

			// Number or ident or delim token
			#[allow(unused_must_use)]
			b'-' => {
				if next.is_digit() {
					self.parse_number(bytes)?
				} else if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | b'_' | b'-')) {
					self.parse_name(bytes)
				} else {
					self.advance(1);
					Token::Delim(b'-')
				}
			}

			// AtRule or Delim token
			b'@' => {
				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | b'_')) {
					self.parse_name(bytes)
				} else {
					self.advance(1);
					Token::Delim(b'@')
				}
			}

			// Ident token
			b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.parse_name(bytes),

			_ => {
				self.advance(1);

				match cur {
					b'\0' => self.next()?,
					b'(' => Token::BracketRoundOpen,
					b')' => Token::BracketRoundClose,
					b'[' => Token::BracketSquareOpen,
					b']' => Token::BracketSquareClose,
					b'{' => Token::BracketCurlyOpen,
					b'}' => Token::BracketCurlyClose,
					b',' => Token::Comma,
					b':' => Token::Colon,
					b';' => Token::Semicolon,

					// Anything else is a delim
					delim => Token::Delim(*delim),
				}
			}
		};

		Ok(token)
	}

	#[inline]
	fn parse_comment(&mut self, bytes: &'a [u8]) -> Result<Token<'a>> {
		// Step over comment opening seq `/*`
		self.advance(2);

		let start = self.pos();

		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if cur == &b'*' {
				let Some(next) = bytes.get(self.pos() + 1) else {
					// EOF
					break;
				};

				if next == &b'/' {
					let res = Token::Comment(&bytes[start..self.pos()]);
					// Step over comment closing seq `*/`
					self.advance(2);

					return Ok(res);
				}
			}

			self.advance(1);
		}

		Err(Error::BadComment)
	}

	#[inline]
	fn parse_name(&mut self, bytes: &'a [u8]) -> Token<'a> {
		let opening = bytes[self.pos()];
		let start = self.pos();

		self.advance(1);

		let mut is_function = false;

		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if !matches!(cur, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
				if cur == &b'(' && opening != b'@' {
					is_function = true;
					// Consume opening paren
					self.advance(1);
				}

				break;
			}

			self.advance(1);
		}

		let bytes = &bytes[start..self.pos()];

		let token = match opening {
			b'@' => Token::AtRule(bytes),
			b'#' => Token::Hash(bytes),
			_ if is_function => Token::Function(bytes),
			_ => Token::Ident(bytes),
		};

		token
	}

	#[inline]
	fn parse_number(&mut self, bytes: &'a [u8]) -> Result<Token<'a>> {
		let start = self.pos();

		let mut cur = unsafe { bytes.get_unchecked(self.pos()) };

		if matches!(cur, b'-' | b'+' | b'.') {
			self.advance(1);
		}

		while !self.is_eof() {
			cur = unsafe { bytes.get_unchecked(self.pos()) };

			// Matches (we don't care about validity): 1px, 1rem, 100%, +110e10, -110, +++++++++1, .1..1
			if !matches!(cur,  b'%' | b'+' | b'-' | b'.' | b'A'..=b'Z'| b'a'..=b'z' |b'0'..=b'9' ) {
				break;
			}

			self.advance(1);
		}

		let bytes = &bytes[start..self.pos()];

		Ok(Token::Number(bytes))
	}

	#[inline]
	fn parse_string(&mut self, bytes: &'a [u8], quote: &u8) -> Result<Token<'a>> {
		let start = self.pos();

		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if cur == quote {
				let token = Token::String(&bytes[start..self.pos()]);
				self.advance(1);
				return Ok(token);
			}

			if cur == &b'\n' {
				self.advance(1);
				// Unescaped newline - parse error
				break;
			}

			if cur == &b'\\' {
				// Ignore any escape seq
				self.advance(1);
			}

			self.advance(1);
		}

		Err(Error::BadString)
	}

	#[inline(always)]
	pub fn peek_next(&mut self) -> Result<Token<'a>> {
		match self.cache {
			Some(token) => Ok(token),
			None => {
				let token = self.next()?;
				self.cache.replace(token);
				self.peek_next()
			}
		}
	}

	#[inline(always)]
	pub fn pos(&self) -> usize {
		self.pos
	}

	#[inline]
	fn skip_whitespace(&mut self, bytes: &'a [u8]) {
		// Step over once
		self.advance(1);

		// Step over until non whitespace
		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if !matches!(cur, b'\n' | b'\r' | b'\t' | b' ') {
				break;
			}

			self.advance(1);
		}
	}

	// #[inline]
}

impl<'a> std::fmt::Debug for Token<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::str::from_utf8_unchecked as str;

		unsafe {
			match self {
				Token::Comment(bytes) => write!(f, "Comment(\"{}\")", str(bytes)),
				Token::Ident(bytes) => write!(f, "Ident(\"{}\")", str(bytes)),
				Token::Function(bytes) => write!(f, "Function(\"{}\")", str(bytes)),
				Token::AtRule(bytes) => write!(f, "AtRule(\"{}\")", str(bytes)),
				Token::Hash(bytes) => write!(f, "Hash(\"{}\")", str(bytes)),
				Token::String(bytes) => write!(f, "String(\"{}\")", str(bytes)),
				Token::Number(bytes) => write!(f, "Number(\"{}\")", str(bytes)),
				Token::Delim(d) => write!(f, "Delim({})", *d as char),
				Token::Whitespace => f.write_str("Whitespace"),
				Token::Colon => f.write_str("Colon"),
				Token::Semicolon => f.write_str("Semicolon"),
				Token::Comma => f.write_str("Comma"),
				Token::BracketRoundOpen => f.write_str("BracketRoundOpen"),
				Token::BracketRoundClose => f.write_str("BracketRoundClose"),
				Token::BracketSquareOpen => f.write_str("BracketSquareOpen"),
				Token::BracketSquareClose => f.write_str("BracketSquareClose"),
				Token::BracketCurlyOpen => f.write_str("BracketCurlyOpen"),
				Token::BracketCurlyClose => f.write_str("BracketCurlyClose"),
			}
		}
	}
}
