use std::hint::unreachable_unchecked;
use std::io::Write;

use self::context::Context;
use self::utils::Helper;

use super::parser::{Error as ParserError, Token, Tokens};
use super::properties::{Descriptor, Trie};

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error<'a> {
	BadComment,
	BadString,
	IO(std::io::Error),
	TooManyLevelsOfIndentation,
	UnexpectedEOF,
	UnexpectedToken(Token<'a>),
	UnexpectedUTF8,
	UnknownProperty,
}

#[derive(Debug)]
pub struct Formatter<'a, T> {
	tokens: Tokens<'a>,
	context: Context<T>,
	prop_trie: Trie<'a>,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

impl<'a, T: std::io::Write> Formatter<'a, T> {
	#[inline]
	pub fn format(&mut self) -> Result<'a, ()> {
		// Top level loop
		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				// Comma and eol
				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.flush()?;
				}

				Token::Comment(bytes) => self.context.write_comment(bytes)?,

				// Selector `div` or `.some-class` or `#some_id`
				Token::Ident(bytes) | Token::Hash(bytes) => self.context.write_all(bytes)?,

				// At-rule `@media ...`, also format it's own block if any
				Token::AtRule(_) => self.format_atrule()?,

				// Any delim `.some_class`
				Token::Delim(delim) => self.context.write_u8(delim)?,

				// Selector `:is(...)`
				Token::Function(_) => self.format_function()?,

				// Selector `:is(...)`
				Token::Colon => self.context.write_u8(b':')?,

				// Selector `[href*="something"]`
				Token::BracketSquareOpen => self.format_attribute_selector()?,

				// Declaration block
				Token::BracketCurlyOpen => self.format_block()?,

				unexpected => return Err(Error::UnexpectedToken(unexpected)),
			}

			match self.tokens.next_with_whitespace() {
				Err(ParserError::EOF) => break,
				Err(err) => {
					// TODO: Should crash or ignore
					eprintln!("Parsing error: {err:?}");
				}

				_ => {}
			};
		}

		if !self.context.is_empty() {
			self.context.flush()?;
		}

		Ok(())
	}

	#[inline]
	fn format_atrule(&mut self) -> Result<'a, ()> {
		todo!()
	}

	#[inline]
	fn format_attribute_selector(&mut self) -> Result<'a, ()> {
		self.whitespace_between_words()?;

		self.context.write_u8(b'[')?;

		loop {
			// No spaces expected
			self.tokens.next()?;

			match self.tokens.current() {
				Token::Comment(bytes) => self.context.write_comment(bytes)?,

				Token::Ident(bytes) => self.context.write_all(bytes)?,

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Delim(del) => self.context.write_u8(del)?,

				Token::BracketSquareClose => {
					self.context.write_u8(b']')?;
					return Ok(());
				}

				unexpected => return Err(Error::UnexpectedToken(unexpected)),
			}
		}
	}

	// CSS now support nesting which means pain in the ass for me
	// We will enforce order of:
	// 1. Declarations like - `background: red;` - separated by newline
	// 2. Nested selectors (if any) like - `&:hover { ... }` - separated by empty line
	// 3. Nested media queries (if any) like - `@media { ... }` - separated by empty line
	#[inline]
	fn format_block(&mut self) -> Result<'a, ()> {
		// Turn `something{` into `something {`, but not `{` to ` {`
		if !self.context.is_empty() {
			self.context.write_space()?;
		}

		self.context.write_u8(b'{')?;
		self.context.flush()?;

		self.context.indent_inc()?;

		// Store all declarations for sorting later
		// Order: 1st
		let mut declarations = Vec::new();

		// Order: 2nd
		let nested_block = Vec::new();
		self.context.layer_push(nested_block);

		// Order: 3nd
		// let at_rules = Vec::new();

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::Comment(bytes) => self.context.write_comment(bytes)?,

				Token::Ident(bytes) if self.context.is_empty() => {
					if let Some(desc) = if bytes.starts_with(b"--") {
						let variable_name = unsafe { std::str::from_utf8_unchecked(bytes) };

						Some(Descriptor::variable(variable_name))
					} else {
						self.prop_trie.get(bytes).copied()
					} {
						let res = self.format_declaration();

						declarations.push((desc, self.context.take()));

						// You can skip trailing `;` in declaration, if next token is `}` ignoring whitespace in between
						if matches!(res, Err(Error::UnexpectedToken(Token::BracketCurlyClose))) {
							continue;
						}

						res?
					} else {
						// CONSIDER: selection sorting ?
						self.context.write_all(bytes)?;
					};
				}

				Token::Function(_) => self.format_function()?,

				Token::AtRule(_) => self.format_atrule()?,

				Token::BracketSquareOpen => self.format_attribute_selector()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				Token::Delim(del) => self.context.write_u8(del)?,

				Token::Colon => self.context.write_u8(b':')?,

				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.flush()?;
				}

				Token::BracketCurlyOpen => self.format_block()?,

				Token::BracketCurlyClose => {
					let buf = unsafe { self.context.layer_take()?.unwrap_unchecked() };

					if !declarations.is_empty() {
						self.write_declarations(declarations)?;

						if !buf.is_empty() {
							self.context.flush()?;
						}
					}

					self.context.current_output().write_all(&buf)?;

					// SAFETY: by recursive nature it's impossible to cause integer underflow
					unsafe { self.context.indent_dec().unwrap_unchecked() };

					self.context.write_u8(b'}')?;

					if matches!(self.tokens.prev(), Some(Token::BracketCurlyOpen)) {
						self.context.write_newline()?;
					}

					if !matches!(self.tokens.peek_next(), Ok(Token::BracketCurlyClose))
					// || matches!(self.tokens.prev(), Some(Token::BracketCurlyOpen))
					{
						dbg!("matches?", self.tokens.peek_next());
						self.context.flush()?;
					}

					self.context.flush()?;

					return Ok(());
				}

				token => return Err(Error::UnexpectedToken(token)),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	#[inline]
	fn format_declaration(&mut self) -> Result<'a, ()> {
		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::BracketCurlyClose => {
					self.context.write_u8(b';')?;
					return Err(Error::UnexpectedToken(Token::BracketCurlyClose));
				}

				Token::Semicolon => {
					self.context.write_u8(b';')?;
					return Ok(());
				}

				Token::Comment(bytes) => self.context.write_comment(bytes)?,

				// `color: var(--some-var);`
				Token::Function(_) => self.format_function()?,

				// `color: #cccccc;`
				Token::Hash(bytes) | Token::Ident(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				// `content: ":)";`
				Token::String(bytes) => self.format_string(bytes)?,

				// `background: var(--some-var), blue;`
				Token::Comma => self.context.write_all(b", ")?,

				// `color: blue;`
				Token::Colon => {
					self.context.write_u8(b':')?;
					self.context.write_space()?;
				}

				unexpected => return Err(Error::UnexpectedToken(unexpected)),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	/// Format CSS function `:is()` or `translate()`
	#[inline]
	pub fn format_function(&mut self) -> Result<'a, ()> {
		let Token::Function(bytes) = self.tokens.current() else {
			// #Safety: Caller must ensure this function is called with valid token
			unsafe { unreachable_unchecked() }
		};

		self.context.write_all(bytes)?;

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::BracketRoundClose => {
					self.context.write_u8(b')')?;
					return Ok(());
				}

				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.write_space()?;
				}

				Token::Comment(bytes) => self.context.write_comment(bytes)?,

				Token::Function(_) => self.format_function()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				Token::Delim(del) => self.context.write_u8(del)?,
				Token::Colon => self.context.write_u8(b':')?,

				Token::BracketSquareOpen => self.format_attribute_selector()?,

				unexpected => return Err(Error::UnexpectedToken(unexpected)),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	#[inline]
	fn format_string(&mut self, bytes: &[u8]) -> Result<'a, ()> {
		self.context.write_u8(b'"')?;
		self.context.write_all(bytes)?;
		self.context.write_u8(b'"')?;

		Ok(())
	}

	#[inline]
	pub fn new(tokens: Tokens<'a>, output: T) -> Self {
		Self {
			tokens,
			context: Context::new(output),
			prop_trie: Trie::new(),
		}
	}

	#[inline]
	fn whitespace_between_words(&mut self) -> Result<'a, ()> {
		let (Some(prev), Ok(next)) = (self.tokens.prev(), self.tokens.peek_next()) else {
			return Ok(());
		};

		if matches!(
			prev,
			Token::BracketRoundClose
				| Token::BracketSquareClose
				| Token::Hash(_)
				| Token::Ident(_)
				| Token::Number(_)
				| Token::Whitespace
		) && matches!(
			next,
			Token::Ident(_)
				| Token::Colon
				| Token::Delim(_)
				| Token::Function(_)
				| Token::Hash(_)
				| Token::Number(_)
		) {
			dbg!(prev, next);
			self.context.write_space()?;
		}

		Ok(())
	}

	fn write_declarations(
		&mut self,
		mut declarations: Vec<(Descriptor<'a>, line::Line)>,
	) -> Result<'a, ()> {
		declarations.sort();

		let mut group = unsafe { declarations.first().unwrap_unchecked() }.0.group();
		for (desc, line) in declarations {
			if desc.group() != group {
				dbg!(group);
				group = desc.group();
				// self.context.write_newline()?;
				self.context.flush()?;
			}

			self.context.flush_line(&line)?;
		}

		Ok(())
	}
}

impl<'a> From<ParserError> for Error<'a> {
	#[inline]
	fn from(value: ParserError) -> Self {
		match value {
			ParserError::BadComment => Error::BadComment,
			ParserError::BadString => Error::BadString,
			ParserError::EOF => Error::UnexpectedEOF,
			ParserError::NonASCII => Error::UnexpectedUTF8,
			// ParserError::NotANumber => ,
		}
	}
}

impl<'a> From<std::io::Error> for Error<'a> {
	#[inline(always)]
	fn from(value: std::io::Error) -> Self {
		Error::IO(value)
	}
}

impl<'a> From<context::IntegerOverflow> for Error<'a> {
	#[inline]
	fn from(_: context::IntegerOverflow) -> Self {
		Error::TooManyLevelsOfIndentation
	}
}

mod context;
mod line;
mod utils;
