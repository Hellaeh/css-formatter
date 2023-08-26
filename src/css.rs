use parser::Parser;
use properties::{Descriptor, Property as CSSProperties};

#[derive(Debug)]
pub struct Error;

pub trait Helper: std::io::Write {
	#[inline(always)]
	fn write_u8(&mut self, byte: u8) -> std::io::Result<()> {
		self.write_all(&[byte])
	}

	#[inline(always)]
	fn write_newline(&mut self) -> std::io::Result<()> {
		self.write_u8(b'\n')
	}

	#[inline(always)]
	fn write_indent(&mut self, indent: usize) -> std::io::Result<()> {
		for _ in 0..indent {
			self.write_u8(b'\t')?;
		}

		Ok(())
	}

	#[inline(always)]
	fn write_newline_with_indent(&mut self, indent: usize) -> std::io::Result<()> {
		self.write_newline()?;
		self.write_indent(indent)
	}
}

impl<T: std::io::Write> Helper for T {}

pub fn format<R: AsRef<str>>(input: R, output: &mut impl std::io::Write) -> std::io::Result<()> {
	let parser = Parser::new(input.as_ref().as_bytes());

	let mut prop_trie = hel_trie::Trie::new();
	// iterate over all CSS properties to populate trie
	for i in 0..std::mem::variant_count::<CSSProperties>() {
		let prop = unsafe { std::mem::transmute::<usize, CSSProperties>(i) };
		let desc = prop.to_descriptor();
		prop_trie.insert(desc.name(), desc);
	}

	let mut indent = 0;
	let mut col = 0;
	let mut prev_token = None;
	// Process tokens
	loop {
		use parser::{Error as ParseError, Token::*};

		let mut token = match parser.next() {
			Ok(Whitespace) => continue,
			Ok(token) => token,
			Err(ParseError::EOF) => break,
			Err(err) => {
				println!("Parsing error: {err:?}");
				continue;
			}
		};

		match token {
			Comment(comment) => {
				// No inline comments
				if col > 0 {
					// Inlined comments should be placed on prev line
					todo!("No inline comments");
					// output.write_newline_with_indent(indentation)?;
				}

				output.write_all(b"/* ")?;
				output.write_all(comment.trim_ascii())?;
				output.write_all(b" */")?;

				output.write_newline()?;
				col = 0;
			}
			// This block should handle most of incoming css
			// All css properties, e.g. `background`, will be processed here
			Ident(ident) => {
				// Add whitespace before ident
				if col > 0 && matches!(prev_token, Some(ref x) if !matches!(x, Delim(_))) {
					output.write_u8(b' ')?;
				}

				// Check if block start with a property
				if prev_token == Some(BracketCurlyOpen)
					&& prop_trie
						.get(unsafe { std::str::from_utf8_unchecked(ident) })
						.is_some()
				{
					// We have to store all properties in a vec in order to sort em
					let mut properties = Vec::new();

					let mut desc = *prop_trie
						.get(unsafe { std::str::from_utf8_unchecked(ident) })
						.expect("Unknown property");
					let mut linebuf = Vec::new();
					linebuf.extend_from_slice(ident);
					prev_token = Some(token);

					loop {
						match parser.peek_next() {
							Ok(BracketCurlyClose) => break,
							Err(ParseError::EOF) => panic!("wtf"),
							Err(_) => todo!("error handling in ident token match"),
							_ => {}
						};

						token = unsafe { parser.next().unwrap_unchecked() };

						match token {
							Whitespace => continue,

							Ident(ident) if linebuf.is_empty() || matches!(prev_token, Some(Comment(_))) => {
								let name = unsafe { std::str::from_utf8_unchecked(ident) };

								if matches!(prev_token, Some(Comment(_))) {
									// Prev line was a comment
									linebuf.write_indent(indent)?;
								}

								if ident.starts_with(b"--") {
									desc = Descriptor::new(name);
								} else {
									desc = *prop_trie
										.get(name)
										.unwrap_or_else(|| panic!("Unknown property: {name}"));
								}

								linebuf.extend_from_slice(ident);
							}

							Ident(bytes) | Number(bytes) | Function(bytes) | Hash(bytes) => {
								if matches!(prev_token, Some(Ident(_))) {
									linebuf.push(b' ');
								}

								linebuf.extend_from_slice(bytes);
							}

							Semicolon => {
								linebuf.extend_from_slice(b";\n");
								properties.push((desc, linebuf.clone()));
								linebuf.clear();
							}

							Comment(bytes) => {
								if !linebuf.is_empty() {
									panic!("Error: no inline comments");
								}

								linebuf.extend_from_slice(b"/* ");
								linebuf.extend_from_slice(bytes.trim_ascii());
								linebuf.extend_from_slice(b" */");
								linebuf.write_newline()?;
							}

							Comma => linebuf.extend_from_slice(b", "),
							Colon => linebuf.extend_from_slice(b": "),

							BracketRoundOpen => linebuf.write_u8(b'(')?,
							BracketRoundClose => linebuf.write_u8(b')')?,

							Delim(del) => linebuf.push(del),

							token => {
								dbg!(token);
								todo!("any other token in properties");
							}
						}

						prev_token = Some(token);
					}

					properties.sort();

					let mut prev_group = properties.first().unwrap().0.group();
					for (desc, line) in properties {
						if desc.group() != prev_group {
							prev_group = desc.group();
							output.write_newline()?;
						}

						output.write_indent(indent)?;
						output.write_all(&line)?;
					}
				} else {
					output.write_all(ident)?;
					col += 1;
				}
			}
			AtRule(rule) => {
				todo!("AtRule")
			}
			Hash(_) => todo!(),
			String(_) => todo!(),
			BadString => if parser.is_eof() {},
			Delim(delim) => {
				if col == 0 {
					output.write_indent(indent)?;
				} else {
					output.write_u8(b' ')?;
				}

				output.write_u8(delim)?;
				col += 1;
			}
			Number(number) => {
				if col > 0 {
					output.write_u8(b' ')?;
				}

				output.write_all(number)?;
				col += 1;
			}
			Colon => {
				output.write_u8(b':')?;
				col += 1;
			}
			Semicolon => {
				output.write_u8(b';')?;
				output.write_newline()?;
				col = 0;
			}
			Comma => {
				output.write_u8(b',')?;
				output.write_newline_with_indent(indent)?;
				col = 0;
			}
			BracketRoundOpen => todo!(),
			BracketRoundClose => todo!(),
			BracketSquareOpen => todo!(),
			BracketSquareClose => todo!(),
			BracketCurlyOpen => {
				if col > 0 {
					output.write_u8(b' ')?;
				}

				output.write_u8(b'{')?;
				output.write_newline()?;

				indent += 1;
				col = 0;
			}
			BracketCurlyClose => {
				indent = indent.checked_sub(1).expect("Format error: curly bracket");

				output.write_indent(indent)?;

				output.write_u8(b'}')?;
				output.write_newline()?;
				output.write_newline()?;
			}
			_ => unreachable!(),
		}

		prev_token = Some(token)
	}

	Ok(())
}

mod parser;
mod properties;
