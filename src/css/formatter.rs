use std::io::Write;

use self::context::Context;
use self::line::{Line, LineWithContext};
use self::utils::Helper;

use super::parser::{Error as ParserError, Parser, Token};
use super::properties::{Descriptor, Property as CSSProperties};

#[derive(Debug)]
pub enum Error<'a> {
	UnexpectedToken(Token<'a>),
	UnexpectedEOF,
	IOError(std::io::Error),
}

#[derive(Debug)]
pub struct Formatter;

impl Formatter {
	#[inline]
	pub fn new() -> Self {
		Self
	}

	#[inline]
	pub fn format<'a>(
		&self,
		parser: &mut Parser,
		output: &mut impl std::io::Write,
	) -> Result<(), Error<'a>> {
		let mut lines = Vec::new();

		let context = Context::new();
		let mut current_line = LineWithContext::new(&context);

		let mut prop_trie = hel_trie::Trie::new();
		// Iterate over all CSS properties to populate trie
		for i in 0..std::mem::variant_count::<CSSProperties>() {
			let prop = unsafe { std::mem::transmute::<usize, CSSProperties>(i) };
			let desc = prop.to_descriptor();
			prop_trie.insert(desc.name(), desc);
		}

		let mut prev_token = None;

		// Top level loop
		loop {
			use Token::*;

			let mut token = match parser.next() {
				Ok(Whitespace) => continue,
				Ok(token) => token,
				Err(ParserError::EOF) => break,
				Err(err) => {
					eprintln!("Parsing error: {err:?}");
					continue;
				}
			};

			match token {
				Comment(bytes) => {
					let line = if !current_line.is_empty() {
						// Inlined comments should be placed on prev line
						let mut temp = LineWithContext::new(&context);
						temp.write_comment(bytes)?;
						temp.take()
					} else {
						current_line.write_comment(bytes)?;
						current_line.end()
					};

					lines.push(line);
				}

				// This block should handle most of incoming css
				// All css properties, e.g. `background`, will be processed here
				Ident(ident) => {
					// Check if block start with a property
					if prev_token == Some(BracketCurlyOpen) && prop_trie.get(ident).is_some() {
						if !current_line.is_empty() {
							todo!("Current line is not empty while property processing");
						}
						current_line.write_all(ident)?;

						// We have to store all properties in a vec to sort later
						let mut properties = Vec::new();

						// Descriptor will be used for sorting
						let mut desc = *prop_trie.get(ident).expect("unknown property");

						prev_token = Some(token);

						// Property loop
						loop {
							match parser.peek_next() {
								Ok(BracketCurlyClose) => {
									if !current_line.is_empty() {
										properties.push((desc, current_line.end()));
									}

									break;
								}
								Err(ParserError::EOF) => panic!("wtf"),
								Err(_) => todo!("error handling in property match"),
								_ => {}
							};

							token = unsafe { parser.next().unwrap_unchecked() };

							match token {
								Whitespace => {
									if current_line.is_empty()
										|| matches!(prev_token, Some(x) if matches!(x, BracketRoundOpen))
									{
										continue;
									}
								}

								Ident(bytes) if current_line.is_empty() => {
									let name = unsafe { std::str::from_utf8_unchecked(bytes) };

									// Check if variable
									if bytes.starts_with(b"--") {
										desc = Descriptor::new(name);
									} else {
										desc = *prop_trie
											.get(name)
											// TODO: change to unknown descriptor in future
											.unwrap_or_else(|| panic!("unknown property: {name}"));
									}

									current_line.write_all(bytes)?;
								}

								Function(bytes) | Hash(bytes) | Ident(bytes) | Number(bytes) => {
									if matches!(prev_token, Some(x) if matches!(x, Colon | Comma | Whitespace)) {
										current_line.write_space()?;
									}

									current_line.write_all(bytes)?;
								}

								Semicolon => {
									current_line.write_u8(b';')?;

									properties.push((desc, current_line.end()));
								}

								Comment(bytes) => {
									// Check if inline comment
									let line = if !current_line.is_empty() {
										// Put comment on prev line
										let mut temp = LineWithContext::new(&context);
										temp.write_comment(bytes)?;
										temp.take()
									} else {
										current_line.write_comment(bytes)?;
										current_line.end()
									};

									properties.push((desc, line));
								}

								Comma => current_line.write_u8(b',')?,
								Colon => current_line.write_u8(b':')?,

								BracketRoundOpen => current_line.write_u8(b'(')?,
								BracketRoundClose => current_line.write_u8(b')')?,

								Delim(del) => {
									if matches!(del, b'#') {
										current_line.write_space()?;
									}

									current_line.write_u8(del)?;
								}

								token => {
									todo!("token in properties: {token:?}");
								}
							}

							prev_token = Some(token);
						}

						// Sort by descriptor and then by line content
						properties.sort();

						let mut prev_group = properties.first().unwrap().0.group();
						for (desc, line) in properties {
							if desc.group() != prev_group {
								prev_group = desc.group();
								lines.push(Line::new())
							}

							lines.push(line)
						}
					} else {
						// Add whitespace before ident
						if !current_line.is_empty() && matches!(prev_token, Some(x) if !matches!(x, Delim(_))) {
							current_line.write_space()?;
						}

						current_line.write_all(ident)?;
					}
				}

				// `@media ...`
				AtRule(rule) => {
					todo!("Process: {token:?}");
				}

				Hash(bytes) => {
					current_line.write_u8(b'#')?;
					current_line.write_all(bytes)?;
				}

				// Selector `[href*="something"]`
				String(bytes) => todo!(),
				BadString => if parser.is_eof() {},

				Delim(delim) => {
					current_line.write_u8(delim)?;
				}

				// Selector - `.abc1` or `#abc`
				Number(bytes) => {
					if !current_line.is_empty() {
						current_line.write_space()?;
					}

					current_line.write_all(bytes)?;
				}

				Colon => {
					current_line.write_u8(b':')?;
				}

				Semicolon => {
					current_line.write_u8(b';')?;
					lines.push(current_line.end())
				}

				Comma => {
					current_line.write_u8(b',')?;
					lines.push(current_line.end())
				}

				// `@media (min-width: 1280px)`
				BracketRoundOpen => {
					if !matches!(prev_token, Some(Function(_))) {
						current_line.write_space()?;
					}

					current_line.write_u8(b'(')?;
				}
				BracketRoundClose => current_line.write_u8(b')')?,

				// `[href*="something"]`
				BracketSquareOpen => current_line.write_u8(b'[')?,
				BracketSquareClose => current_line.write_u8(b']')?,

				BracketCurlyOpen => {
					if !current_line.is_empty() {
						current_line.write_space()?;
					}

					current_line.write_u8(b'{')?;
					lines.push(current_line.end());

					context.indentation().inc();
				}

				BracketCurlyClose => {
					context
						.indentation()
						.dec()
						.map_err(|_| Error::UnexpectedToken(BracketCurlyClose))?;

					current_line.write_u8(b'}')?;
					lines.push(current_line.end());
					// Empty line
					lines.push(Line::new())
				}

				_ => unreachable!(),
			}

			prev_token = Some(token)
		}

		for line in lines {
			output.write_all(&line)?;
			output.write_newline()?;
		}

		Ok(())
	}
}

impl<'a> From<ParserError> for Error<'a> {
	#[inline]
	fn from(value: ParserError) -> Self {
		match value {
			ParserError::CommentEOF => todo!(),
			ParserError::EOF => todo!(),
			ParserError::NonASCII => todo!(),
			ParserError::NotANumber => todo!(),
		}
	}
}

impl<'a> From<std::io::Error> for Error<'a> {
	#[inline]
	fn from(value: std::io::Error) -> Self {
		Error::IOError(value)
	}
}

mod context;
mod line;
mod utils;
