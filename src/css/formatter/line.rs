use crate::consts::ASCII;

use super::Helper;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug)]
pub struct Line {
	buf: Vec<u8>,
}

/// Hardcoded max length of single line. Shouldn't include indentation.
/// If `Line > MAX_LENGTH`, split into multiple lines.
pub const MAX_LENGTH: usize = 80;

impl Line {
	#[inline]
	pub fn flush_self_with_indent(
		&mut self,
		indent: u8,
		output: &mut impl std::io::Write,
	) -> std::io::Result<u32> {
		debug_assert!(!self.is_empty());

		let wrote = if self.len() > MAX_LENGTH {
			let mut count = 0;

			for (offset, line) in Self::split(self, 0) {
				output.finish_line_with_indent(line, indent + offset)?;

				count += 1;
			}

			count
		} else {
			output.finish_line_with_indent(self, indent)?;

			1
		};

		self.clear();

		Ok(wrote)
	}

	#[inline]
	pub fn new() -> Self {
		Self {
			buf: Vec::with_capacity(MAX_LENGTH),
		}
	}

	/// Split any ASCII byte array into smaller chunks.
	///
	/// Quite expensive, but most of CSS should fit in [`Self::MAX_LENGTH`]
	#[inline]
	fn split(buf: &[u8], offset: u8) -> impl Iterator<Item = (u8, &[u8])> {
		// We need vec in order to backtrack
		// TODO: improve algorithm, remove vec
		let mut lines = Vec::new();

		fn split<'a>(buf: &'a [u8], offset: u8, lines: &mut Vec<(u8, &'a [u8])>) -> usize {
			let mut i = 0;
			let mut prev = i;

			while i < buf.len() {
				match buf[i] {
					// We can't split string
					ASCII::DOUBLE_QUOTE => {
						i += 1;

						while buf[i] != b'"' {
							i += 1;
						}
					}

					ASCII::PAREN_OPEN => {
						// We might step out of this block
						let mut start = prev;

						i += 1;

						lines.push((offset, &buf[start..i]));

						start = i;

						let mut level = 0;
						let len_before = lines.len();

						// Find matching paren
						while i < buf.len() {
							match buf[i] {
								ASCII::PAREN_CLOSE if level == 0 => break,

								ASCII::PAREN_OPEN => level += 1,
								ASCII::PAREN_CLOSE => level -= 1,

								_ => {}
							}

							i += 1;
						}

						// FIXME: fml
						if (i - start) < (MAX_LENGTH / 4) {
							lines.truncate(len_before - 1);
							continue;
						}

						let inner = &buf[start..i];

						let res = split(inner, offset + 1, lines);

						// FIXME: do a flip
						if (lines.len() - len_before) < 2 {
							lines.truncate(len_before - 1);
						} else {
							i = start + res;
							prev = i;
						}
					}

					ASCII::COLON if matches!(buf.get(i + 1).copied(), Some(ASCII::SPACE)) => {
						lines.push((offset, &buf[prev..=i]));

						// Skip space
						i += 2;

						let inner = &buf[i..];

						i += split(inner, offset + 1, lines);
						prev = i;
					}

					ASCII::COMMA if offset > 0 => {
						i += 1;

						lines.push((offset, &buf[prev..i]));

						// Skip space
						prev = i + 1;
					}

					_ => {}
				}

				i += 1;
			}

			let last = &buf[prev..];

			if !last.is_empty() {
				lines.push((offset, last));
			}

			i
		}

		split(buf, offset, &mut lines);

		lines.into_iter()
	}
}

impl std::ops::Deref for Line {
	type Target = Vec<u8>;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.buf
	}
}

impl std::ops::DerefMut for Line {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.buf
	}
}

#[cfg(test)]
mod tests {
	use super::Line;

	#[test]
	fn string_splitting() {
		// For clippy
		type Offset = u8;
		type ByteArray = [u8];

		let cases: &[(&ByteArray, &[(Offset, &ByteArray)])] = &[
			(
				br#"a:is([href*="path1"]:not([href~="path3"]), [href*="path2"]:not([href~="path4"]))"#,
				&[
					(0, b"a:is("),
					(1, br#"[href*="path1"]:not([href~="path3"]),"#),
					(1, br#"[href*="path2"]:not([href~="path4"])"#),
					(0, b")"),
				],
			),
			(
				b"background: conic-gradient(from 230deg at 51.63% 52%, rgb(36, 0, 255) 0deg, rgb(0, 135, 255) 65deg, rgb(154, 25, 246) 198.75deg, rgb(15, 33, 192) 255deg, rgb(84, 135, 229) 300deg, rgb(108, 49, 226) 360deg);",
				&[
					(0, b"background:"),
					(1, b"conic-gradient("),
					(2, b"from 230deg at 51.63% 52%,"),
					(2, b"rgb(36, 0, 255) 0deg,"),
					(2, b"rgb(0, 135, 255) 65deg,"),
					(2, b"rgb(154, 25, 246) 198.75deg,"),
					(2, b"rgb(15, 33, 192) 255deg,"),
					(2, b"rgb(84, 135, 229) 300deg,"),
					(2, b"rgb(108, 49, 226) 360deg"),
					(1, b");"),
				],
			),
		];

		for (before, after) in cases.iter().copied() {
			let res = Line::split(before, 0);
			let mut count = 0;

			for (res, should_be) in res.zip(after.iter().copied()) {
				use std::str::from_utf8_unchecked as to_str;

				unsafe {
					let left = to_str(res.1);
					let right = to_str(should_be.1);

					assert_eq!(left, right);

					assert_eq!(res.0, should_be.0, "Wrong offset for {left}");
				}

				count += 1;
			}

			if count != after.len() {
				panic!("something went wrong");
			}
		}
	}
}
