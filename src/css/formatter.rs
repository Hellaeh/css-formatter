use std::hint::unreachable_unchecked;
use std::io::Write;

use self::context::Context;
use self::utils::Helper;

use super::parser::{Error as ParserError, Token, Tokens};
use super::properties::{Descriptor, Trie};

macro_rules! unexpected_token {
	($token: expr) => {
		return Err(Error::UnexpectedToken {
			token: $token,
			line: line!(),
		})
	};
}

#[derive(Debug)]
#[allow(clippy::upper_case_acronyms)]
pub enum Error<'a> {
	BadComment,
	BadString,
	IO(std::io::Error),
	TooManyLevelsOfIndentation,
	UnexpectedEOF,
	UnexpectedToken { token: Token<'a>, line: u32 },
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
		// Write top level comment
		// WARN: For now the only place comments allowed, also remove this comment, when support arrives
		while let Token::Comment(bytes) = self.tokens.current() {
			self.context.write_comment(bytes)?;
			self.tokens.next()?;
		}

		// Top level loop
		loop {
			match self.tokens.current() {
				Token::Comment(_) => self.format_comment()?,

				// Start of a ruleset (hopefully)
				Token::BracketSquareOpen
				| Token::Colon
				| Token::Delim(_)
				| Token::Hash(_)
				| Token::Ident(_) => self.format_ruleset()?,

				// At-rule `@media ...`, also format it's own block if any
				Token::AtRule(_) => self.format_atrule()?,

				token => unexpected_token!(token),
			}

			match self.tokens.next() {
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

	/// Caller must ensure that current token is a CSS at-rule
	#[inline]
	fn format_atrule(&mut self) -> Result<'a, ()> {
		let Token::AtRule(at_rule) = self.tokens.current() else {
			unsafe { unreachable_unchecked() }
		};

		self.context.write_all(at_rule)?;

		self.context.write_space()?;

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::Comment(_) => self.format_comment()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				// `@import url("something")`
				Token::Function(_) => self.format_function()?,

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Delim(del) => self.context.write_u8(del)?,

				Token::Semicolon => {
					self.context.write_u8(b';')?;
					self.context.flush()?;

					if matches!(
						self.tokens.peek_next_with_whitespace(),
						Ok(Token::Whitespace)
					) {
						unsafe { self.tokens.next_with_whitespace().unwrap_unchecked() };
					}

					if self.tokens.peek_next_with_whitespace().is_ok() {
						self.context.flush()?;
					}

					return Ok(());
				}

				Token::Colon => {
					self.context.write_u8(b':')?;
					self.context.write_space()?;

					self.tokens.next()?;

					continue;
				}

				Token::BracketRoundOpen => self.context.write_u8(b'(')?,
				Token::BracketRoundClose => self.context.write_u8(b')')?,

				Token::BracketCurlyOpen => {
					self.format_block()?;

					return Ok(());
				}

				token => unexpected_token!(token),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	#[inline]
	fn format_attribute_selector(&mut self) -> Result<'a, ()> {
		self.context.write_u8(b'[')?;

		// No spaces expected
		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Comment(_) => self.format_comment()?,

				Token::Ident(bytes) => self.context.write_all(bytes)?,

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Delim(del) => self.context.write_u8(del)?,

				Token::BracketSquareClose => {
					self.context.write_u8(b']')?;
					break;
				}

				token => unexpected_token!(token),
			}

			self.tokens.next()?;
		}

		Ok(())
	}

	#[inline]
	fn format_block(&mut self) -> Result<'a, ()> {
		if !self.context.is_empty() {
			self.context.write_space()?;
		}

		self.context.write_u8(b'{')?;
		self.context.flush()?;

		self.context.indent_inc()?;

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::Comment(_) => self.format_comment()?,

				Token::BracketCurlyOpen
				| Token::BracketSquareOpen
				| Token::Colon
				| Token::Delim(b'.')
				| Token::Delim(b'&')
				| Token::Hash(_)
				| Token::Ident(_) => self.format_ruleset()?,

				// `@keyframes { 0% { color: red; }}`
				Token::Number(bytes) => self.context.write_all(bytes)?,

				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.flush()?;
				}

				Token::BracketCurlyClose => {
					unsafe { self.context.indent_dec().unwrap_unchecked() }

					self.context.write_u8(b'}')?;
					self.context.flush()?;

					// Skip next whitespace token
					if matches!(
						self.tokens.peek_next_with_whitespace(),
						Ok(Token::Whitespace)
					) {
						unsafe { self.tokens.next_with_whitespace().unwrap_unchecked() };
					}

					// Add empty line after block, if there's more content
					if !matches!(
						self.tokens.peek_next_with_whitespace(),
						Ok(Token::BracketCurlyClose)
					) {
						self.context.flush()?;
					}

					break;
				}

				_ => {}
			}

			self.tokens.next_with_whitespace()?;
		}

		Ok(())
	}

	// FIXME: Remove inline `always`, once function actually do something
	#[inline(always)]
	fn format_comment(&self) -> Result<'a, ()> {
		unexpected_token!(self.tokens.current())
	}

	#[inline]
	fn format_declaration(&mut self) -> Result<'a, ()> {
		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::BracketCurlyClose => {
					self.context.write_u8(b';')?;

					unexpected_token!(Token::BracketCurlyClose)
				}

				Token::Semicolon => {
					self.context.write_u8(b';')?;
					return Ok(());
				}

				Token::Comment(_) => self.format_comment()?,

				// `color: var(--some-var);`
				Token::Function(_) => self.format_function()?,

				// `color: #cccccc;`
				Token::Hash(bytes) | Token::Ident(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				Token::Delim(del) => self.context.write_u8(del)?,

				// `content: ":)";`
				Token::String(bytes) => self.format_string(bytes)?,

				// `background: var(--some-var), blue;`
				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.write_space()?;

					self.tokens.next()?;

					continue;
				}

				// `color: blue;`
				Token::Colon => {
					self.context.write_u8(b':')?;
					self.context.write_space()?;

					self.tokens.next()?;

					continue;
				}

				token => unexpected_token!(token),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	// CSS now support nesting which means pain in the ass for me
	// We will enforce order of:
	// 1. Declarations like - `background: red;` - separated by newline
	// 2. Nested selectors (if any) like - `&:hover { ... }` - separated by empty line
	// 3. Nested media queries (if any) like - `@media { ... }` - separated by empty line
	#[inline]
	fn format_declaration_block(&mut self) -> Result<'a, ()> {
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
		let current_layer = Vec::new();
		self.context.layer_push(current_layer);

		// Order: 3nd
		// TODO: ??
		// let at_rules = Vec::new();

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::Comment(_) => self.format_comment()?,

				Token::Ident(bytes) if self.context.is_empty() => {
					let desc = self.get_description(bytes);

					let res = self.format_declaration();

					declarations.push((desc, self.context.take()));

					// You can skip trailing `;` in declaration, if next token is `}` ignoring whitespace in between
					if matches!(
						res,
						Err(Error::UnexpectedToken {
							token: Token::BracketCurlyClose,
							..
						})
					) {
						continue;
					}

					res?;
				}

				Token::BracketSquareOpen
				| Token::Colon
				| Token::Delim(b'.')
				| Token::Delim(b'&')
				| Token::Hash(_)
				| Token::Ident(_) => self.format_ruleset()?,

				// Token::Function(_) => self.format_function()?,
				Token::AtRule(_) => self.format_atrule()?,

				Token::Delim(del) => self.context.write_u8(del)?,

				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.flush()?;
				}

				Token::BracketCurlyOpen => self.format_declaration_block()?,

				Token::BracketCurlyClose => {
					let buf = unsafe { self.context.layer_pop().unwrap_unchecked() };

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
					self.context.flush()?;

					// Skip next whitespace token
					if matches!(
						self.tokens.peek_next_with_whitespace(),
						Ok(Token::Whitespace)
					) {
						unsafe { self.tokens.next_with_whitespace().unwrap_unchecked() };
					}

					// Add empty line after block, if there's more content
					if !matches!(
						self.tokens.peek_next_with_whitespace(),
						Ok(Token::BracketCurlyClose)
					) {
						self.context.flush()?;
					}

					break;
				}

				token => unexpected_token!(token),
			}

			self.tokens.next_with_whitespace()?;
		}

		Ok(())
	}

	/// Format CSS function `:is()` or `translate()`
	#[inline]
	pub fn format_function(&mut self) -> Result<'a, ()> {
		let Token::Function(bytes) = self.tokens.current() else {
			// #Safety: Caller must ensure this function is called with valid token
			unsafe { unreachable_unchecked() }
		};

		let mut level = 0;

		self.context.write_all(bytes)?;

		self.tokens.next()?;

		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::BracketRoundOpen => {
					self.context.write_u8(b'(')?;

					level += 1;
				}

				Token::BracketRoundClose => {
					self.context.write_u8(b')')?;

					if level == 0 {
						return Ok(());
					} else {
						level -= 1;
					}
				}

				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.write_space()?;

					self.tokens.next()?;

					continue;
				}

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Comment(_) => self.format_comment()?,

				Token::Function(_) => self.format_function()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				Token::Delim(del @ b'.') => {
					self.context.write_u8(del)?;

					let next = self.tokens.next()?;

					if !matches!(next, Token::Ident(_)) {
						unexpected_token!(next);
					}

					continue;
				}

				Token::Delim(del) => self.process_delim(del)?,

				Token::Colon => {
					self.context.write_u8(b':')?;

					self.tokens.next()?;

					continue;
				}

				Token::BracketSquareOpen => self.format_attribute_selector()?,

				token => unexpected_token!(token),
			}

			self.tokens.next_with_whitespace()?;
		}
	}

	#[inline]
	fn format_ruleset(&mut self) -> Result<'a, ()> {
		loop {
			match self.tokens.current() {
				Token::Whitespace => self.whitespace_between_words()?,

				Token::Comment(_) => self.format_comment()?,

				Token::Ident(bytes) | Token::Hash(bytes) => self.context.write_all(bytes)?,

				Token::Delim(del @ b'*') => self.context.write_u8(del)?,

				Token::Delim(del @ b'.') => {
					self.context.write_u8(del)?;

					let next = self.tokens.next()?;

					if !matches!(next, Token::Ident(_)) {
						unexpected_token!(next)
					}

					continue;
				}

				Token::Delim(del @ b'&') => {
					self.context.write_u8(del)?;

					if self.tokens.next_with_whitespace()? == Token::Whitespace {
						self.context.write_space()?;
						self.tokens.next()?;
					}

					continue;
				}

				Token::Colon => {
					self.context.write_u8(b':')?;

					match self.tokens.next()? {
						Token::Colon | Token::Ident(_) => continue,
						Token::Function(_) => self.format_function()?,
						token => {
							dbg!(self.tokens.prev(), self.tokens.next()?);
							unexpected_token!(token);
						}
					}
				}

				Token::Delim(del) => self.process_delim(del)?,

				// Comma means EOL for us
				Token::Comma => {
					self.context.write_u8(b',')?;
					self.context.flush()?;
				}

				// Format block and return
				Token::BracketCurlyOpen => {
					self.format_declaration_block()?;
					break;
				}

				// Token::Function(_) => todo!(),
				Token::BracketSquareOpen => self.format_attribute_selector()?,

				token => {
					dbg!(self.tokens.prev(), self.tokens.next()?);
					unexpected_token!(token);
				}
			}

			self.tokens.next_with_whitespace()?;
		}

		Ok(())
	}

	#[inline]
	fn format_string(&mut self, bytes: &[u8]) -> Result<'a, ()> {
		self.context.write_u8(b'"')?;
		self.context.write_all(bytes)?;
		self.context.write_u8(b'"')?;

		Ok(())
	}

	fn get_description(&self, bytes: &'a [u8]) -> Descriptor<'a> {
		let name = unsafe { std::str::from_utf8_unchecked(bytes) };

		if bytes.len() > 1 && bytes[0] == b'-' {
			return if bytes[1] == b'-' {
				Descriptor::variable(name)
			} else {
				Descriptor::unknown(name)
			};
		}

		if let Some(desc) = self.prop_trie.get(bytes).copied() {
			return desc;
		}

		Descriptor::unknown(name)
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
	fn process_delim(&mut self, delim: u8) -> Result<'a, ()> {
		match delim {
			b'+' | b'-' | b'>' | b'~' | b'*' | b'/' => {
				self.context.write_space()?;
				self.context.write_u8(delim)?;
				self.context.write_space()?;
			}

			b'#' => {
				if self.tokens.prev() == Some(Token::Whitespace) {
					self.context.write_space()?;
				}

				self.context.write_u8(delim)?;

				if matches!(
					self.tokens.peek_next_with_whitespace(),
					Ok(Token::Whitespace)
				) {
					self.context.write_space()?;
				}
			}

			_ => {
				dbg!(&self.context);
				unexpected_token!(self.tokens.current());
			}
		}

		Ok(())
	}

	#[inline]
	fn whitespace_between_words(&mut self) -> Result<'a, ()> {
		let (Some(prev), Ok(next)) = (self.tokens.prev(), self.tokens.peek_next_with_whitespace())
		else {
			return Ok(());
		};

		if matches!(
			prev,
			Token::BracketRoundClose
				| Token::BracketSquareClose
				| Token::AtRule(_)
				| Token::Colon
				| Token::Hash(_)
				| Token::Ident(_)
				| Token::Number(_)
		) && matches!(
			next,
			Token::BracketRoundOpen
				| Token::BracketSquareOpen
				| Token::AtRule(_)
				| Token::Colon
				| Token::Delim(_)
				| Token::Function(_)
				| Token::Hash(_)
				| Token::Ident(_)
				| Token::Number(_)
		) {
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
				group = desc.group();
				self.context.flush()?;
			}

			self.context.write_all(&line)?;
			self.context.flush()?;
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
