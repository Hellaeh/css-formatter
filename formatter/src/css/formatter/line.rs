use crate::css::formatter::line::splitter::{Split, Splitter};

use super::Helper;

#[derive(Clone, Eq, Ord, PartialEq, PartialOrd, Debug, Default)]
pub struct Line {
	buf: Vec<u8>,
}

/// Hardcoded max length of single line
/// If `Line > MAX_LENGTH`, split into multiple lines
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

			for Split { offset, bytes } in Splitter::split(self) {
				output.finish_line_with_indent(bytes, indent + offset)?;

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

mod splitter;
