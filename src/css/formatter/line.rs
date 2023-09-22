use super::Helper;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
		let mut lines = Vec::new();

		let mut i = 0;
		let mut prev = i;

		while i < buf.len() {
			match buf[i] {
				b'(' => {
					let mut level = 0;

					lines.push((offset, &buf[prev..=i]));

					i += 1;
					prev = i;

					while i < buf.len() {
						match buf[i] {
							b'(' => level += 1,
							b')' => {
								if level == 0 {
									lines.push((offset + 1, &buf[prev..i]));
									prev = i;
									break;
								}

								level -= 1;
							}
							// Force split on comma
							b',' if level == 0 => {
								let inner = &buf[prev..=i];

								// Skip space
								i += 2;
								prev = i;

								if inner.len() > MAX_LENGTH {
									lines.extend(Self::split(inner, offset + 1));
								} else {
									lines.push((offset + 1, inner))
								}
							}
							_ => {}
						}

						i += 1;
					}
				}

				b':' if matches!(buf.get(i + 1), Some(b' ')) => {
					let mut level = 0;

					lines.push((offset, &buf[prev..=i]));

					i += 2;
					prev = i;

					while i < buf.len() {
						match buf[i] {
							b'(' => level += 1,
							b')' => level -= 1,

							b',' if level == 0 => {
								let inner = &buf[prev..=i];

								// Skip space
								i += 2;
								prev = i;

								if inner.len() > MAX_LENGTH {
									lines.extend(Self::split(inner, offset + 1));
								} else {
									lines.push((offset + 1, inner));
								}
							}
							_ => {}
						}

						i += 1;
					}
				}

				_ => {}
			}

			i += 1;
		}

		lines.push((offset, &buf[prev..]));

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
		let cases: &[(&[u8], &[(u8, &[u8])])] = &[
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
				b"background: \
				conic-gradient(\
				from 230deg at 51.63% 52%, \
				rgb(36, 0, 255) 0deg, \
				rgb(0, 135, 255) 65deg, \
				rgb(154, 25, 246) 198.75deg, \
				rgb(15, 33, 192) 255deg, \
				rgb(84, 135, 229) 300deg, \
				rgb(108, 49, 226) 360deg\
				);",
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
