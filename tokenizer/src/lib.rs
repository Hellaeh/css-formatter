// // use crate::utils::Helper;

// use consts::ASCII;

// use token::Token;

// #[derive(Debug)]
// #[allow(clippy::upper_case_acronyms)]
// pub enum Error {
// 	BadComment,
// 	BadString,
// 	EOF,
// 	NonASCII,
// }

// /// An opinionated parser for opinionated CSS formatter
// #[derive(Debug)]
// pub struct Tokenizer<'a> {
// 	buf: &'a [u8],
// 	// Current position (index)
// 	pos: usize,
// }

// pub type Result<T> = std::result::Result<T, Error>;

// impl<'a> Tokenizer<'a> {
// 	#[inline(always)]
// 	pub fn advance(&mut self, amount: usize) {
// 		self.pos += amount
// 	}

// 	#[inline(always)]
// 	pub fn is_eof(&self) -> bool {
// 		self.pos() >= self.buf.len()
// 	}

// 	#[inline]
// 	pub fn new(input: &'a [u8]) -> Self {
// 		Self { buf: input, pos: 0 }
// 	}

// 	#[inline(always)]
// 	fn get_current_byte(&self) -> u8 {
// 		debug_assert!(!self.is_eof());
// 		unsafe { *self.buf.get_unchecked(self.pos()) }
// 	}

// 	#[inline(always)]
// 	fn peek_next_byte(&self) -> Option<u8> {
// 		self.buf.get(self.pos() + 1).copied()
// 	}

// 	#[inline]
// 	pub fn next(&mut self) -> Result<Token<'a>> {
// 		let bytes = self.buf;

// 		if self.is_eof() {
// 			return Err(Error::EOF);
// 		}

// 		let cur = self.get_current_byte();
// 		let next = self.peek_next_byte();

// 		if !cur.is_ascii() {
// 			return Err(Error::NonASCII);
// 		}

// 		let token = match cur {
// 			// A comment or delim token
// 			ASCII::SLASH => {
// 				if next == Some(ASCII::ASTERISK) {
// 					self.parse_comment(bytes)?
// 				} else {
// 					self.advance(1);
// 					Token::Delim(ASCII::SLASH)
// 				}
// 			}

// 			// Whitespace token
// 			ASCII::SPACE | ASCII::TAB | ASCII::LF | ASCII::CR => {
// 				self.skip_whitespace(bytes);
// 				Token::Whitespace
// 			}

// 			// A hash or delim token
// 			ASCII::HASH => {
// 				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | ASCII::UNDERSCORE)) {
// 					self.parse_name(bytes)
// 				} else {
// 					self.advance(1);
// 					Token::Delim(ASCII::HASH)
// 				}
// 			}

// 			// String token
// 			quote_type @ (ASCII::SINGLE_QUOTE | ASCII::DOUBLE_QUOTE) => {
// 				// Step over opening quote
// 				self.advance(1);
// 				self.parse_string(bytes, quote_type)?
// 			}

// 			// Number token
// 			b'0'..=b'9' => self.parse_number(bytes)?,

// 			// Number or delim token
// 			#[allow(unused_must_use)]
// 			delim @ (ASCII::FULL_STOP | ASCII::PLUS) => {
// 				if next.is_digit()
// 					|| (next == Some(ASCII::FULL_STOP)
// 						&& matches!(bytes.get(self.pos() + 2), Some(x) if x.is_digit()))
// 				{
// 					self.parse_number(bytes)?
// 				} else {
// 					self.advance(1);
// 					Token::Delim(delim)
// 				}
// 			}

// 			// Number or ident or delim token
// 			#[allow(unused_must_use)]
// 			ASCII::DASH => {
// 				if next.is_digit()
// 					|| (next == Some(ASCII::FULL_STOP)
// 						&& matches!(bytes.get(self.pos() + 2), Some(x) if x.is_digit()))
// 				{
// 					self.parse_number(bytes)?
// 				} else if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | ASCII::UNDERSCORE | ASCII::DASH))
// 				{
// 					self.parse_name(bytes)
// 				} else {
// 					self.advance(1);
// 					Token::Delim(ASCII::DASH)
// 				}
// 			}

// 			// AtRule or Delim token
// 			ASCII::AT => {
// 				if matches!(next, Some(x) if matches!(x, b'a'..=b'z' | b'A'..=b'Z' | ASCII::UNDERSCORE)) {
// 					self.parse_name(bytes)
// 				} else {
// 					self.advance(1);
// 					Token::Delim(ASCII::AT)
// 				}
// 			}

// 			// Ident token
// 			b'a'..=b'z' | b'A'..=b'Z' | ASCII::UNDERSCORE => self.parse_name(bytes),

// 			_ => {
// 				self.advance(1);

// 				match cur {
// 					b'\0' => self.next()?,
// 					ASCII::PAREN_OPEN => Token::BracketRoundOpen,
// 					ASCII::PAREN_CLOSE => Token::BracketRoundClose,
// 					ASCII::SQUARED_OPEN => Token::BracketSquareOpen,
// 					ASCII::SQUARED_CLOSE => Token::BracketSquareClose,
// 					ASCII::CURLY_OPEN => Token::BracketCurlyOpen,
// 					ASCII::CURLY_CLOSE => Token::BracketCurlyClose,
// 					ASCII::COMMA => Token::Comma,
// 					ASCII::COLON => Token::Colon,
// 					ASCII::SEMICOLON => Token::Semicolon,

// 					// Anything else is a delim
// 					delim => Token::Delim(delim),
// 				}
// 			}
// 		};

// 		Ok(token)
// 	}

// 	#[inline]
// 	fn parse_comment(&mut self, bytes: &'a [u8]) -> Result<Token<'a>> {
// 		// Step over comment opening seq `/*`
// 		self.advance(2);

// 		let start = self.pos();

// 		while !self.is_eof() {
// 			let cur = self.get_current_byte();

// 			if cur == ASCII::ASTERISK {
// 				let Some(next) = bytes.get(self.pos() + 1).copied() else {
// 					// EOF
// 					break;
// 				};

// 				if next == ASCII::SLASH {
// 					let res = Token::Comment(&bytes[start..self.pos()]);
// 					// Step over comment closing seq `*/`
// 					self.advance(2);

// 					return Ok(res);
// 				}
// 			}

// 			self.advance(1);
// 		}

// 		Err(Error::BadComment)
// 	}

// 	#[inline]
// 	fn parse_name(&mut self, bytes: &'a [u8]) -> Token<'a> {
// 		let opening = bytes[self.pos()];
// 		let start = self.pos();

// 		self.advance(1);

// 		let mut is_function = false;

// 		while !self.is_eof() {
// 			let cur = self.get_current_byte();

// 			if !matches!(cur, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | ASCII::DASH | ASCII::UNDERSCORE) {
// 				if cur == ASCII::PAREN_OPEN && opening != ASCII::AT {
// 					is_function = true;
// 					// Consume opening paren
// 					self.advance(1);
// 				}

// 				break;
// 			}

// 			self.advance(1);
// 		}

// 		let bytes = &bytes[start..self.pos()];

// 		let token = match opening {
// 			ASCII::AT => Token::AtRule(bytes),
// 			ASCII::HASH => Token::Hash(bytes),
// 			_ if is_function => Token::Function(bytes),
// 			_ => Token::Ident(bytes),
// 		};

// 		token
// 	}

// 	#[inline]
// 	fn parse_number(&mut self, bytes: &'a [u8]) -> Result<Token<'a>> {
// 		let start = self.pos();

// 		let mut cur = self.get_current_byte();

// 		if matches!(cur, ASCII::DASH | ASCII::PLUS | ASCII::FULL_STOP) {
// 			self.advance(1);
// 		}

// 		while !self.is_eof() {
// 			cur = self.get_current_byte();

// 			// Matches (we don't care about validity): 1px, 1rem, 100%, +110e10, -110, +++++++++1, .1..1
// 			if !matches!(cur,  ASCII::PERCENTAGE| ASCII::PLUS | ASCII::DASH | ASCII::FULL_STOP | b'A'..=b'Z'| b'a'..=b'z' |b'0'..=b'9' )
// 			{
// 				break;
// 			}

// 			self.advance(1);
// 		}

// 		let bytes = &bytes[start..self.pos()];

// 		Ok(Token::Number(bytes))
// 	}

// 	#[inline]
// 	fn parse_string(&mut self, bytes: &'a [u8], quote: u8) -> Result<Token<'a>> {
// 		let start = self.pos();

// 		while !self.is_eof() {
// 			let cur = self.get_current_byte();

// 			if cur == quote {
// 				let token = Token::String(&bytes[start..self.pos()]);
// 				self.advance(1);
// 				return Ok(token);
// 			}

// 			if matches!(cur, ASCII::LF | ASCII::CR) {
// 				self.advance(1);
// 				// Unescaped newline - parse error
// 				break;
// 			}

// 			if cur == ASCII::BACKSLASH {
// 				// Ignore any escape seq
// 				self.advance(1);
// 			}

// 			self.advance(1);
// 		}

// 		Err(Error::BadString)
// 	}

// 	#[inline(always)]
// 	pub fn pos(&self) -> usize {
// 		self.pos
// 	}

// 	#[inline]
// 	fn skip_whitespace(&mut self, bytes: &'a [u8]) {
// 		// Step over once
// 		self.advance(1);

// 		// Step over until non whitespace
// 		while !self.is_eof() {
// 			let cur = self.get_current_byte();

// 			if !matches!(cur, ASCII::LF | ASCII::CR | ASCII::TAB | ASCII::SPACE) {
// 				break;
// 			}

// 			self.advance(1);
// 		}
// 	}

// 	// #[inline]
// }

// impl<'a> std::fmt::Debug for Token<'a> {
// 	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
// 		use std::str::from_utf8_unchecked as str;

// 		unsafe {
// 			match self {
// 				Token::Comment(bytes) => write!(f, "Comment(\"{}\")", str(bytes)),
// 				Token::Ident(bytes) => write!(f, "Ident(\"{}\")", str(bytes)),
// 				Token::Function(bytes) => write!(f, "Function(\"{}\")", str(bytes)),
// 				Token::AtRule(bytes) => write!(f, "AtRule(\"{}\")", str(bytes)),
// 				Token::Hash(bytes) => write!(f, "Hash(\"{}\")", str(bytes)),
// 				Token::String(bytes) => write!(f, "String(\"{}\")", str(bytes)),
// 				Token::Number(bytes) => write!(f, "Number(\"{}\")", str(bytes)),
// 				Token::Delim(d) => write!(f, "Delim({})", *d as char),
// 				Token::Whitespace => f.write_str("Whitespace"),
// 				Token::Colon => f.write_str("Colon"),
// 				Token::Semicolon => f.write_str("Semicolon"),
// 				Token::Comma => f.write_str("Comma"),
// 				Token::BracketRoundOpen => f.write_str("BracketRoundOpen"),
// 				Token::BracketRoundClose => f.write_str("BracketRoundClose"),
// 				Token::BracketSquareOpen => f.write_str("BracketSquareOpen"),
// 				Token::BracketSquareClose => f.write_str("BracketSquareClose"),
// 				Token::BracketCurlyOpen => f.write_str("BracketCurlyOpen"),
// 				Token::BracketCurlyClose => f.write_str("BracketCurlyClose"),
// 			}
// 		}
// 	}
// }

// mod cache;
// mod token;

// pub use cache::Cache;
