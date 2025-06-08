#![feature(portable_simd)]
#![feature(test)]

pub use token::Token;

use consts::ASCII;
use simd::tokenize_comment_simd;
use utils::ByteHelper;

pub const LANE_WIDTH: usize = 16;
pub const LANE_WIDTH_MASK: usize = LANE_WIDTH - 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error {
	BadComment,
	BadString,
	EOF,
	NonASCII,
}

macro_rules! pat {
	// alpha
	(A) => {
		b'A'..=b'Z' | b'a'..=b'z'
	};
	// numeric
	(N) =>  {
		b'0'..=b'9'
	};
	// alphanumeric
	(AN) => {
		pat!(A) | pat!(N)
	};
}

#[derive(Debug)]
// pub struct Tokenizer<const LANE_WIDTH: usize = 16> { // this is not supported i guess?
pub struct Tokenizer {
	cursor: *const u8,
	eof: *const u8, // this would be a null terminator position in C
}

pub type Result = std::result::Result<Token, Error>;

// impl<const LANE_WIDTH: usize> Tokenizer<LANE_WIDTH> {
impl Tokenizer {
	#[inline(always)]
	pub fn advance(&mut self, steps: usize) {
		self.cursor = unsafe { self.cursor.add(steps) };
	}

	#[inline(always)]
	pub fn is_eof(&self) -> bool {
		self.cursor >= self.eof
	}

	#[inline(always)]
	pub fn rem(&self) -> usize {
		self.eof.addr() - self.cursor.addr()
	}

	#[inline]
	pub fn new(input: *const [u8]) -> Self {
		const {
			assert!(LANE_WIDTH.is_power_of_two());
			assert!(LANE_WIDTH >= 16);
			assert!(LANE_WIDTH <= 64);
		}

		let ptr = input as *const u8;

		assert!(
			ptr.addr() & LANE_WIDTH_MASK == 0,
			"pointer is not aligned to width - {}",
			{ LANE_WIDTH }
		);

		Self {
			cursor: ptr,
			eof: unsafe { ptr.add(input.len()) },
		}
	}

	#[inline(always)]
	fn peek(&self, i: usize) -> u8 {
		unsafe { *self.cursor.add(i) }
	}

	#[inline(always)]
	fn try_peek(&self, i: usize) -> Option<u8> {
		if self.rem() > i {
			return Some(self.peek(i));
		}

		None
	}

	#[inline(always)]
	fn current_byte(&self) -> u8 {
		debug_assert!(!self.is_eof());

		unsafe { *self.cursor }
	}

	#[inline(always)]
	#[must_use]
	fn try_peek_next_byte(&self) -> Option<u8> {
		self.try_peek(1)
	}

	#[inline]
	pub fn next_token(&mut self) -> Result {
		if self.is_eof() {
			return Err(Error::EOF);
		}

		let ch = self.current_byte();
		let next_ch = self.try_peek_next_byte();

		if !ch.is_ascii() {
			return Err(Error::NonASCII);
		}

		let token = match ch {
			// A comment or delim token
			ASCII::SLASH => {
				if next_ch == Some(ASCII::ASTERISK) {
					self.tokenize_comment()?
				} else {
					self.advance(1);
					Token::Delim(ASCII::SLASH)
				}
			}

			// Whitespace token
			ASCII::SPACE | ASCII::TAB | ASCII::LF | ASCII::CR => self.tokenize_whitespace(),

			// A hash or delim token
			ASCII::HASH => {
				if matches!(next_ch, Some(x) if matches!(x, pat!(A) | ASCII::UNDERSCORE)) {
					self.tokenize_name()
				} else {
					self.advance(1);
					Token::Delim(ASCII::HASH)
				}
			}

			// String token
			ASCII::SINGLE_QUOTE | ASCII::QUOTE => self.tokenize_string()?,

			// Number token
			pat!(N) => self.tokenize_number(),

			// Number or delim token
			delim @ (ASCII::FULL_STOP | ASCII::PLUS) => {
				if next_ch.is_digit()
					|| (matches!(next_ch, Some(ASCII::FULL_STOP))
						&& matches!(self.try_peek(2), Some(x) if x.is_digit()))
				{
					self.tokenize_number()
				} else {
					self.advance(1);
					Token::Delim(delim)
				}
			}

			// Number or ident or delim token
			ASCII::DASH => {
				if next_ch.is_digit()
					|| (next_ch == Some(ASCII::FULL_STOP)
						&& matches!(self.try_peek(2), Some(x) if x.is_digit()))
				{
					self.tokenize_number()
				} else if matches!(next_ch, Some(x) if matches!(x, pat!(A) | ASCII::UNDERSCORE | ASCII::DASH))
				{
					self.tokenize_name()
				} else {
					self.advance(1);
					Token::Delim(ASCII::DASH)
				}
			}

			// AtRule or Delim token
			ASCII::AT => {
				if matches!(next_ch, Some(x) if matches!(x, pat!(A) | ASCII::UNDERSCORE)) {
					self.tokenize_name()
				} else {
					self.advance(1);
					Token::Delim(ASCII::AT)
				}
			}

			// Ident token
			pat!(A) | ASCII::UNDERSCORE => self.tokenize_name(),

			_ => {
				self.advance(1);

				match ch {
					// null bytes are valid in utf8
					b'\0' => self.next_token()?,

					ASCII::PAREN_OPEN => Token::BracketRoundOpen,
					ASCII::PAREN_CLOSE => Token::BracketRoundClose,
					ASCII::SQUARED_OPEN => Token::BracketSquareOpen,
					ASCII::SQUARED_CLOSE => Token::BracketSquareClose,
					ASCII::CURLY_OPEN => Token::BracketCurlyOpen,
					ASCII::CURLY_CLOSE => Token::BracketCurlyClose,
					ASCII::COMMA => Token::Comma,
					ASCII::COLON => Token::Colon,
					ASCII::SEMICOLON => Token::Semicolon,

					// Anything else is a delim
					delim => Token::Delim(delim),
				}
			}
		};

		Ok(token)
	}

	#[inline]
	fn tokenize_comment(&mut self) -> Result {
		// Step over comment opening seq `/*`
		self.advance(2);

		let token = tokenize_comment_simd(self);

		// Step over comment closing seq `*/`
		self.advance(2);

		token
	}

	#[inline]
	fn tokenize_name(&mut self) -> Token {
		let opening = self.current_byte();
		let ptr = self.cursor;

		// first character is already processed at this point
		self.advance(1);

		let mut is_function = false;

		while !self.is_eof() {
			let ch = self.current_byte();

			if !matches!(ch, pat!(AN) | ASCII::DASH | ASCII::UNDERSCORE) {
				if ch == ASCII::PAREN_OPEN && opening != ASCII::AT {
					is_function = true;
					// Consume opening paren
					self.advance(1);
				}

				break;
			}

			self.advance(1);
		}

		let variant = match opening {
			ASCII::AT => Token::AtRule,
			ASCII::HASH => Token::Hash,
			_ if is_function => Token::Function,
			_ => Token::Ident,
		};

		Token::from(variant, ptr, self.cursor)
	}

	#[inline]
	fn tokenize_number(&mut self) -> Token {
		let ptr = self.pos();
		let ch = self.current_byte();

		if matches!(ch, ASCII::DASH | ASCII::PLUS | ASCII::FULL_STOP) {
			self.advance(1);
		}

		while !self.is_eof() {
			let ch = self.current_byte();

			// Matches (we don't care about validity): 1px, 1rem, 100%, +110e10, -110, +++++++++1, .1..1
			if !matches!(
				ch,
				ASCII::PERCENTAGE | ASCII::PLUS | ASCII::DASH | ASCII::FULL_STOP | pat!(AN)
			) {
				break;
			}

			self.advance(1);
		}

		Token::from(Token::Number, ptr, self.cursor)
	}

	#[inline]
	fn tokenize_string(&mut self) -> Result {
		// Step over opening quote
		self.advance(1);

		let opening_quote = self.current_byte();
		let ptr = self.cursor;

		while !self.is_eof() {
			let ch = self.current_byte();

			if ch == opening_quote {
				let token = Token::from(Token::String, ptr, self.cursor);
				self.advance(1);
				return Ok(token);
			}

			if matches!(ch, ASCII::LF | ASCII::CR) {
				self.advance(1);
				// Unescaped newline - parse error
				break;
			}

			if ch == ASCII::BACKSLASH {
				// Ignore any escape seq
				self.advance(1);
			}

			self.advance(1);
		}

		Err(Error::BadString)
	}

	#[inline(always)]
	pub fn pos(&self) -> *const u8 {
		self.cursor
	}

	#[inline]
	fn tokenize_whitespace(&mut self) -> Token {
		let mut is_newline = false;

		while !self.is_eof() {
			let ch = self.current_byte();

			match ch {
				ASCII::LF | ASCII::CR => is_newline = true,
				ASCII::TAB | ASCII::SPACE => {}
				_ => break,
			}

			self.advance(1);
		}

		Token::Whitespace(is_newline)
	}
}

impl Iterator for Tokenizer {
	type Item = Result;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		match self.next_token() {
			ok @ Ok(_) => Some(ok),
			err @ Err(inner) => match inner {
				Error::EOF => None,
				_ => Some(err),
			},
		}
	}
}

mod simd;
mod span;
mod token;
pub mod utils;
