use crate::utils::Helper;

#[derive(Clone, Copy, PartialEq)]
pub enum Token<'a> {
	Comment(&'a [u8]),

	Ident(&'a [u8]),
	Function(&'a [u8]),
	AtRule(&'a [u8]),
	Hash(&'a [u8]),
	String(&'a [u8]),
	BadString,
	// URL(&'a [u8]), // Function
	// BadURL, // Not supported
	Delim(u8),
	Number(&'a [u8]),
	// Percentage, // Number is good enough for formatter
	// Dimension, // Same
	Whitespace,
	// CDO, // Not supported
	// CDC, // Not supported
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
pub enum Error {
	CommentEOF,
	EOF,
	NonASCII,
	NotANumber,
}

/// An opinionated parser for opinionated CSS formatter
/// Support only ASCII
#[derive(Debug)]
pub struct Parser<'a> {
	buf: &'a [u8],
	// Current position (index)
	pos: usize,
	// For `peek`ing
	cache: Option<Token<'a>>,
}

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
	pub fn next(&mut self) -> Result<Token<'a>, Error> {
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
			b' ' | b'\t' | b'\n' => {
				self.process_whitespace(bytes);
				Token::Whitespace
			}

			// A hash or delim token
			b'#' => {
				self.advance(1);

				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | b'_')) {
					self.parse_name(bytes, &b'#')
				} else {
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
					self.parse_name(bytes, &0)
				} else {
					self.advance(1);
					Token::Delim(b'-')
				}
			}

			// AtRule or Delim token
			b'@' => {
				self.advance(1);

				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | b'_')) {
					self.parse_name(bytes, &b'@')
				} else {
					Token::Delim(b'@')
				}
			}

			// Ident token
			b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.parse_name(bytes, &0),

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
	fn parse_comment(&mut self, bytes: &'a [u8]) -> Result<Token<'a>, Error> {
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

		Err(Error::CommentEOF)
	}

	#[inline]
	fn parse_name(&mut self, bytes: &'a [u8], opening: &u8) -> Token<'a> {
		let start = self.pos();

		let mut is_function = false;

		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if !matches!(cur, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_') {
				if matches!(cur, b'(' | b')') {
					is_function = true;
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
	fn parse_number(&mut self, bytes: &'a [u8]) -> Result<Token<'a>, Error> {
		let start = self.pos();

		let mut cur = unsafe { bytes.get_unchecked(self.pos()) };
		let mut opening = false;

		if matches!(cur, b'-' | b'+' | b'.') {
			self.advance(1);
			opening = true;
		}

		while !self.is_eof() {
			cur = unsafe { bytes.get_unchecked(self.pos()) };

			if !matches!(cur, b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-') {
				break;
			}

			self.advance(1);
		}

		let bytes = &bytes[start..self.pos()];

		if bytes.len() <= (opening as usize) {
			return Err(Error::NotANumber);
		}

		Ok(Token::Number(bytes))
	}

	#[inline]
	fn parse_string(&mut self, bytes: &'a [u8], quote: &u8) -> Result<Token<'a>, Error> {
		// [`BadString`] instead of [`String`] in case EOF found
		let mut token = Token::BadString;

		let start = self.pos();

		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if cur == quote {
				token = Token::String(&bytes[start..self.pos()]);
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

		Ok(token)
	}

	#[inline(always)]
	pub fn peek_next(&mut self) -> Result<Token, Error> {
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
	fn process_whitespace(&mut self, bytes: &'a [u8]) -> Token<'a> {
		// Step over once
		self.advance(1);

		// Step over until non whitespace
		while !self.is_eof() {
			let cur = unsafe { bytes.get_unchecked(self.pos()) };

			if !matches!(cur, b' ' | b'\t' | b'\n') {
				break;
			}

			self.advance(1);
		}

		Token::Whitespace
	}

	// #[inline]
}

impl<'a> std::fmt::Debug for Token<'a> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::str::from_utf8_unchecked as str;
		use Token::*;

		f.write_str("Token: ")?;

		unsafe {
			match self {
				Comment(bytes) => write!(f, "Comment({})", str(bytes)),
				Ident(bytes) => write!(f, "Ident({})", str(bytes)),
				Function(bytes) => write!(f, "Function({})", str(bytes)),
				AtRule(bytes) => write!(f, "AtRule({})", str(bytes)),
				Hash(bytes) => write!(f, "Hash({})", str(bytes)),
				String(bytes) => write!(f, "String({})", str(bytes)),
				Number(bytes) => write!(f, "Number({})", str(bytes)),
				Delim(d) => write!(f, "Delim({d})"),
				BadString => f.write_str("BadString"),
				Whitespace => f.write_str("Whitespace"),
				Colon => f.write_str("Color"),
				Semicolon => f.write_str("Semicolon"),
				Comma => f.write_str("Comma"),
				BracketRoundOpen => f.write_str("BracketRoundOpen"),
				BracketRoundClose => f.write_str("BracketRoundClose"),
				BracketSquareOpen => f.write_str("BracketSquareOpen"),
				BracketSquareClose => f.write_str("BracketSquareClose"),
				BracketCurlyOpen => f.write_str("BracketCurlyOpen"),
				BracketCurlyClose => f.write_str("BracketCurlyClose"),
			}
		}
	}
}
