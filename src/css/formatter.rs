use std::hint::unreachable_unchecked;
use std::io::Write;

use crate::consts::ASCII;

use super::parser::{Cache, Error as ParserError};
use super::properties::{Descriptor, Trie};
use super::tokens::Token;

use self::context::Context;
use self::utils::Helper;

macro_rules! unexpected_token {
	($token: expr, $self: expr) => {{
		#[cfg(debug_assertions)]
		{
			eprintln!("--------------------------------------------------");
			eprintln!("{:?}", $self.context);
			eprintln!("{:?}", $self.token_cache);
			eprintln!("--------------------------------------------------");
		}

		return Err(Error::UnexpectedToken {
			token: $token,
			line: line!(),
		});
	}};
}

macro_rules! debug_unexpected_token {
	($token: expr, $self: expr) => {{
		#[cfg(debug_assertions)]
		{
			unexpected_token!($token, $self)
		}
	}};
}

macro_rules! ruleset_start_seq {
	() => {
		Token::BracketSquareOpen
			| Token::Colon
			| Token::Delim(ASCII::AMPERSAND)
			| Token::Delim(ASCII::ASTERISK)
			| Token::Delim(ASCII::FULL_STOP)
			| Token::Hash(_)
			| Token::Ident(_)
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
}

#[derive(Debug)]
pub struct Formatter<'a, T> {
	token_cache: Cache<'a>,
	context: Context<T>,
	props: Trie<'a>,
}

pub type Result<'a, T> = std::result::Result<T, Error<'a>>;

impl<'a, T: std::io::Write> Formatter<'a, T> {
	#[inline]
	pub fn format(&mut self) -> Result<'a, ()> {
		// Write top level comment
		// FIXME: For now the only place comments allowed, also remove this comment, when support arrives
		while let Token::Comment(bytes) = self.token_cache.current() {
			self.context.write_comment(bytes)?;
			self.context.flush()?;
			self.context.flush()?;

			self.token_cache.next()?;
		}

		// Top level loop
		loop {
			match self.token_cache.current() {
				Token::Comment(_) => self.format_comment()?,

				// Start of a ruleset (hopefully)
				ruleset_start_seq!() => self.format_ruleset()?,

				// At-rule `@media ...`, also format it's own block if any
				Token::AtRule(_) => self.format_atrule()?,

				token => unexpected_token!(token, self),
			}

			match self.token_cache.next() {
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
		let Token::AtRule(at_rule) = self.token_cache.current() else {
			debug_unexpected_token!(self.token_cache.current(), self);

			unsafe { std::hint::unreachable_unchecked() };
		};

		self.context.write_all(at_rule)?;
		self.context.write_space()?;

		self.token_cache.next()?;

		loop {
			match self.token_cache.current() {
				Token::Whitespace => self.process_whitespace()?,

				Token::Comment(_) => self.format_comment()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				// `@import url("something")`
				Token::Function(_) => self.format_function()?,

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Delim(del) => self.process_delim(del)?,

				Token::Semicolon => {
					self.context.write_u8(ASCII::SEMICOLON)?;
					self.context.flush()?;

					if self.token_cache.peek_next().is_ok() {
						self.context.flush()?;
					}

					return Ok(());
				}

				Token::Colon => {
					self.context.write_u8(ASCII::COLON)?;
					self.context.write_space()?;

					self.token_cache.next()?;

					continue;
				}

				Token::BracketRoundOpen => self.context.write_u8(ASCII::PAREN_OPEN)?,
				Token::BracketRoundClose => self.context.write_u8(ASCII::PAREN_CLOSE)?,

				Token::BracketCurlyOpen => {
					// Hack??
					if self.context.indent() == 0 || at_rule == b"@keyframes" {
						self.format_block()?;
					} else {
						self.format_declaration_block()?;
					}

					break;
				}

				token => unexpected_token!(token, self),
			}

			self.token_cache.next_with_whitespace()?;
		}

		Ok(())
	}

	/// Syntax: [`attribute`( `operator` `value` (`char`)?)?]
	#[inline]
	fn format_attribute_selector(&mut self) -> Result<'a, ()> {
		self.context.write_u8(ASCII::SQUARED_OPEN)?;

		// attr
		{
			let Token::Ident(bytes) = self.token_cache.next()? else {
				unexpected_token!(self.token_cache.current(), self);
			};

			self.context.write_all(bytes)?;

			if let Token::BracketSquareClose = self.token_cache.peek_next()? {
				unsafe { self.token_cache.next().unwrap_unchecked() };
				self.context.write_u8(ASCII::SQUARED_CLOSE)?;

				return Ok(());
			}
		}

		// operator might be composite `*=` or simple `=`
		{
			let Token::Delim(del) = self.token_cache.next()? else {
				unexpected_token!(self.token_cache.current(), self);
			};

			self.context.write_u8(del)?;

			if let Token::Delim(del) = self.token_cache.peek_next()? {
				unsafe { self.token_cache.next().unwrap_unchecked() };
				self.context.write_u8(del)?;
			}
		}

		// value
		{
			let Token::String(bytes) = self.token_cache.next()? else {
				unexpected_token!(self.token_cache.current(), self);
			};

			self.format_string(bytes)?;
		}

		// `i` or `I` or `s` or `S`
		{
			if let Token::Ident(bytes) = self.token_cache.peek_next()? {
				unsafe { self.token_cache.next().unwrap_unchecked() };
				self.context.write_space()?;
				self.context.write_all(bytes)?;
			}
		}

		// closing `]`
		{
			let Token::BracketSquareClose = self.token_cache.next()? else {
				unexpected_token!(self.token_cache.current(), self);
			};

			self.context.write_u8(ASCII::SQUARED_CLOSE)?;
		}

		Ok(())
	}

	#[inline]
	fn format_block(&mut self) -> Result<'a, ()> {
		if !self.context.is_empty() {
			self.context.write_space()?;
		}

		self.context.write_u8(ASCII::CURLY_OPEN)?;
		self.context.flush()?;

		self.context.indent_inc()?;

		self.token_cache.next()?;

		loop {
			match self.token_cache.current() {
				Token::Whitespace => self.process_whitespace()?,

				Token::Comment(_) => self.format_comment()?,

				Token::AtRule(_) => self.format_atrule()?,

				Token::BracketCurlyOpen
				| Token::BracketSquareOpen
				| Token::Colon
				| Token::Delim(ASCII::FULL_STOP)
				| Token::Delim(ASCII::AMPERSAND)
				| Token::Hash(_)
				| Token::Ident(_) => self.format_ruleset()?,

				// `@keyframes { 0% { color: red; }}`
				Token::Number(bytes) => self.context.write_all(bytes)?,

				Token::Comma => {
					self.context.write_u8(ASCII::COMMA)?;
					self.context.flush()?;
				}

				Token::BracketCurlyClose => {
					unsafe { self.context.indent_dec().unwrap_unchecked() }

					self.context.write_u8(ASCII::CURLY_CLOSE)?;
					self.context.flush()?;

					// Add empty line after block, if there's more content
					if !matches!(self.token_cache.peek_next(), Ok(Token::BracketCurlyClose)) {
						self.context.flush()?;
					}

					break;
				}

				token => unexpected_token!(token, self),
			}

			self.token_cache.next_with_whitespace()?;
		}

		Ok(())
	}

	// FIXME: Remove inline `always`, once function actually do something
	#[inline(always)]
	fn format_comment(&self) -> Result<'a, ()> {
		unexpected_token!(self.token_cache.current(), self)
	}

	#[inline]
	fn format_declaration(&mut self) -> Result<'a, ()> {
		let Token::Ident(bytes) = self.token_cache.current() else {
			debug_unexpected_token!(self.token_cache.current(), self);

			unsafe { unreachable_unchecked() };
		};

		self.context.write_all(bytes)?;

		let Token::Colon = self.token_cache.next()? else {
			unexpected_token!(self.token_cache.current(), self);
		};

		self.context.write_u8(ASCII::COLON)?;

		self.token_cache.next()?;

		loop {
			match self.token_cache.current() {
				Token::Comment(_) => self.format_comment()?,

				// Trailing `;` is optional
				Token::BracketCurlyClose => {
					self.context.write_u8(ASCII::SEMICOLON)?;

					unexpected_token!(Token::BracketCurlyClose, self)
				}

				// `;`
				Token::Semicolon => {
					self.context.write_u8(ASCII::SEMICOLON)?;

					break;
				}

				// `content: ":)";`
				Token::String(bytes) => {
					self.context.write_space()?;
					self.format_string(bytes)?;
				}

				// `color: var(--some-var);` or `background: conic-gradient(...)`
				Token::Function(_) => {
					self.context.write_space()?;
					self.format_function()?;
				}

				// `color: #cccccc;`
				Token::Hash(bytes) | Token::Ident(bytes) | Token::Number(bytes) => {
					self.context.write_space()?;
					self.context.write_all(bytes)?
				}

				Token::Delim(del @ ASCII::HASH) => {
					self.context.write_space()?;
					self.process_delim(del)?;
				}
				Token::Delim(del) => self.process_delim(del)?,

				// `background: var(--some-var), blue;`
				Token::Comma => {
					self.context.write_u8(ASCII::COMMA)?;

					self.token_cache.next()?;

					continue;
				}

				token => unexpected_token!(token, self),
			}

			// self.token_cache.next_with_whitespace()?;
			self.token_cache.next()?;
		}

		Ok(())
	}

	// CSS now support nesting which means pain in the ass for me
	// We will enforce order of:
	// 1. Declarations like - `background: red;` - separated by newline
	// 2. Nested selectors or at-rules (if any) like - `&:hover { ... }` - separated by empty line
	#[inline]
	fn format_declaration_block(&mut self) -> Result<'a, ()> {
		// Turn `something{` into `something {`, but not `{` to ` {`
		if !self.context.is_empty() {
			self.context.write_space()?;
		}

		self.context.write_u8(ASCII::CURLY_OPEN)?;
		self.context.flush()?;

		self.context.indent_inc()?;

		// Store all declarations for sorting later
		// Order: 1st
		let mut declarations = Vec::new();

		// Order: 2nd
		let current_layer = Vec::new();
		self.context.layer_push(current_layer);

		self.token_cache.next()?;

		loop {
			match self.token_cache.current() {
				// This is where we return
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

					self.context.write_u8(ASCII::CURLY_CLOSE)?;
					self.context.flush()?;

					// Add empty line after block, if there's more content
					if !matches!(self.token_cache.peek_next(), Ok(Token::BracketCurlyClose)) {
						self.context.flush()?;
					}

					break;
				}

				Token::Comment(_) => self.format_comment()?,

				// Declaration: `background: blue;` or `color: green;`
				// Not: `div&` or `div &`
				Token::Ident(bytes)
					if self.context.is_empty()
						&& !matches!(
							self.token_cache.peek_next()?,
							Token::Delim(ASCII::AMPERSAND)
						) =>
				{
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

				// Nested CSS ruleset
				ruleset_start_seq!() => self.format_ruleset()?,

				// Nested CSS at-rule
				Token::AtRule(_) => self.format_atrule()?,

				Token::Delim(del) => self.process_delim(del)?,

				token => unexpected_token!(token, self),
			}

			self.token_cache.next()?;
		}

		Ok(())
	}

	/// Format CSS function `:is()` or `translate()`
	#[inline]
	fn format_function(&mut self) -> Result<'a, ()> {
		let Token::Function(bytes) = self.token_cache.current() else {
			debug_unexpected_token!(self.token_cache.current(), self);

			// #Safety: Caller must ensure this function is called with valid token
			unsafe { unreachable_unchecked() }
		};

		self.context.write_all(bytes)?;

		self.token_cache.next()?;

		let mut level = 0;
		loop {
			match self.token_cache.current() {
				Token::Whitespace => self.process_whitespace()?,

				Token::BracketRoundOpen => {
					self.context.write_u8(ASCII::PAREN_OPEN)?;

					level += 1;
				}

				Token::BracketRoundClose => {
					self.context.write_u8(ASCII::PAREN_CLOSE)?;

					if level == 0 {
						break;
					} else {
						level -= 1;
					}
				}

				Token::Comma => {
					self.context.write_u8(ASCII::COMMA)?;
					self.context.write_space()?;
				}

				Token::String(bytes) => self.format_string(bytes)?,

				Token::Comment(_) => self.format_comment()?,

				// Nested functions: `max(calc(...), min(...))`
				Token::Function(_) => self.format_function()?,

				Token::Ident(bytes) | Token::Hash(bytes) | Token::Number(bytes) => {
					self.context.write_all(bytes)?
				}

				Token::Delim(del) => self.process_delim(del)?,

				// Nested selectors: `:has(:is(...))`
				Token::Colon => self.format_pseudo()?,

				Token::BracketSquareOpen => self.format_attribute_selector()?,

				token => unexpected_token!(token, self),
			}

			self.token_cache.next_with_whitespace()?;
		}

		Ok(())
	}

	#[inline]
	fn format_pseudo(&mut self) -> Result<'a, ()> {
		self.context.write_u8(ASCII::COLON)?;

		match self.token_cache.next()? {
			// `::before`
			Token::Colon => {
				self.context.write_u8(ASCII::COLON)?;

				match self.token_cache.next()? {
					// `::before`
					Token::Ident(bytes) => self.context.write_all(bytes)?,

					// `::part(...)`
					Token::Function(_) => self.format_function()?,

					token => unexpected_token!(token, self),
				}
			}

			// Pseudo-class: `:active` or `:hover`
			Token::Ident(bytes) => self.context.write_all(bytes)?,

			// Preudo-class: `:is(...)` or `:has(...)`
			Token::Function(_) => self.format_function()?,

			token => unexpected_token!(token, self),
		};

		Ok(())
	}

	#[inline]
	fn format_ruleset(&mut self) -> Result<'a, ()> {
		loop {
			match self.token_cache.current() {
				// Format block and return
				Token::BracketCurlyOpen => {
					self.format_declaration_block()?;
					break;
				}

				Token::Whitespace => self.process_whitespace()?,

				Token::Comment(_) => self.format_comment()?,

				// `p` or `div` or `#some-id`
				Token::Ident(bytes) | Token::Hash(bytes) => self.context.write_all(bytes)?,

				// `:has()` or `::before`
				Token::Colon => self.format_pseudo()?,

				// Selector: `*` or `*::before`
				Token::Delim(del @ ASCII::ASTERISK) => self.context.write_u8(del)?,
				Token::Delim(del) => self.process_delim(del)?,

				// Comma means EOL for us
				Token::Comma => {
					self.context.write_u8(ASCII::COMMA)?;
					self.context.flush()?;
				}

				// Selector: [href*="something"]
				Token::BracketSquareOpen => self.format_attribute_selector()?,

				token => unexpected_token!(token, self),
			}

			self.token_cache.next_with_whitespace()?;
		}

		Ok(())
	}

	#[inline]
	fn format_string(&mut self, bytes: &[u8]) -> Result<'a, ()> {
		self.context.write_u8(ASCII::QUOTE)?;
		self.context.write_all(bytes)?;
		self.context.write_u8(ASCII::QUOTE)?;

		Ok(())
	}

	#[inline]
	fn get_description(&self, bytes: &'a [u8]) -> Descriptor<'a> {
		let name = unsafe { std::str::from_utf8_unchecked(bytes) };

		if bytes.len() > 1 && bytes[0] == ASCII::DASH {
			return if bytes[1] == ASCII::DASH {
				Descriptor::variable(name)
			} else {
				Descriptor::unknown(name)
			};
		}

		if let Some(desc) = self.props.get(bytes).copied() {
			return desc;
		}

		Descriptor::unknown(name)
	}

	#[inline]
	pub fn new(cache: Cache<'a>, output: T) -> Self {
		Self {
			token_cache: cache,
			context: Context::new(output),
			props: Trie::new(),
		}
	}

	#[inline]
	fn process_delim(&mut self, delim: u8) -> Result<'a, ()> {
		match delim {
			ASCII::ASTERISK
			| ASCII::DASH
			| ASCII::FORWARD_SLASH
			| ASCII::GT
			| ASCII::PLUS
			| ASCII::TILDE => {
				if !self.context.is_empty() && !matches!(self.context.last(), Some(b' ')) {
					self.context.write_space()?;
				}

				self.context.write_u8(delim)?;
				// FIXME: better space handling?
				self.context.write_space()?;
			}

			// `!important`
			ASCII::EXCLAMATION => {
				self.context.write_space()?;
				self.context.write_u8(ASCII::EXCLAMATION)?;

				let Token::Ident(bytes) = self.token_cache.next()? else {
					unexpected_token!(self.token_cache.current(), self);
				};

				self.context.write_all(bytes)?;
			}

			// Nested selector: `& .parent {` or `.parent & {` or `& + div {`
			ASCII::AMPERSAND => {
				match self.token_cache.next()? {
					// Space is mandatory for: `& div` or `& custom-element`
					Token::Ident(bytes) => {
						self.context.write_u8(delim)?;
						self.context.write_space()?;
						self.context.write_all(bytes)?;
					}

					Token::Delim(ASCII::FULL_STOP) | Token::Colon | Token::Hash(_) => {
						if unsafe {
							self
								.token_cache
								.peek_prev_with_whitespace()
								.unwrap_unchecked()
						} != Token::Whitespace
						{
							self.context.write_u8(delim)?;
						}

						match self.token_cache.current() {
							Token::Delim(_) => self.process_delim(ASCII::FULL_STOP)?,
							Token::Colon => self.format_pseudo()?,
							Token::Hash(bytes) => self.context.write_all(bytes)?,
							_ => unsafe { unreachable_unchecked() },
						}
					}

					Token::Delim(ASCII::AMPERSAND) => {
						self.context.write_u8(delim)?;

						if unsafe {
							self
								.token_cache
								.peek_prev_with_whitespace()
								.unwrap_unchecked()
						} == Token::Whitespace
						{
							self.context.write_space()?;
						}

						self.process_delim(delim)?;
					}

					// Remove `&` for `& [+>~] whatever`
					Token::Delim(del) => self.process_delim(del)?,

					// `& {` or `&& {` or `.parent & {`
					Token::BracketCurlyOpen => {
						self.token_cache.prev();

						if !self.context.is_empty()
							&& unsafe {
								self
									.token_cache
									.peek_prev_with_whitespace()
									.unwrap_unchecked()
							} == Token::Whitespace
						{
							self.context.write_space()?;
						}

						self.context.write_u8(delim)?;
					}

					Token::BracketRoundClose => {
						self.token_cache.prev();
						self.context.write_u8(delim)?;
					}

					_ => unexpected_token!(self.token_cache.current(), self),
				}
			}

			// Class selector `.some-class`
			ASCII::FULL_STOP => {
				self.context.write_u8(delim)?;

				let Token::Ident(bytes) = self.token_cache.next()? else {
					unexpected_token!(self.token_cache.current(), self);
				};

				self.context.write_all(bytes)?;
			}

			ASCII::HASH => {
				self.context.write_u8(delim)?;

				let Token::Number(bytes) = self.token_cache.next()? else {
					unexpected_token!(self.token_cache.current(), self);
				};

				self.context.write_all(bytes)?;
			}

			_ => unexpected_token!(self.token_cache.current(), self),
		}

		Ok(())
	}

	#[inline]
	fn process_whitespace(&mut self) -> Result<'a, ()> {
		let (Some(prev), Ok(next)) = (
			self.token_cache.peek_prev_with_whitespace(),
			self.token_cache.peek_next(),
		) else {
			return Ok(());
		};

		if matches!(
			prev,
			Token::AtRule(_)
				| Token::BracketSquareClose
				| Token::Colon
				| Token::Function(_)
				| Token::Hash(_)
				| Token::Ident(_)
				| Token::Number(_)
				| Token::String(_)
				| Token::BracketRoundClose
		) && matches!(
			next,
			Token::AtRule(_)
				| Token::BracketSquareOpen
				| Token::Colon
				| Token::Delim(ASCII::FULL_STOP)
				| Token::Function(_)
				| Token::Hash(_)
				| Token::Ident(_)
				| Token::Number(_)
				| Token::String(_)
				| Token::BracketRoundOpen
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
