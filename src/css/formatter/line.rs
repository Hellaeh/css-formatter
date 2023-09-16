use super::Helper;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Line {
	buf: Vec<u8>,
}

impl Line {
	/// Hardcoded max length of single line.
	/// If `self.len() > MAX_LENGTH`, split into multiple lines.
	const MAX_LENGTH: usize = 100;

	#[inline]
	pub fn new() -> Self {
		Self {
			buf: Vec::with_capacity(Self::MAX_LENGTH),
		}
	}

	#[inline(always)]
	pub fn flush_self_with_indent(
		&mut self,
		indent: u8,
		output: &mut impl std::io::Write,
	) -> std::io::Result<u32> {
		let lines = Self::split(self, indent);

		output.write_indent(indent)?;
		output.write_all(self)?;
		output.write_newline()?;

		self.clear();

		Ok(lines)
	}

	/// Inserts `\t(n + 1)\n` at certain breakpoints where `n` is indent
	/// Quite expensive
	#[inline]
	fn split(buf: &mut Vec<u8>, indent: u8) -> u32 {
		if buf.len() <= Self::MAX_LENGTH {
			return 1;
		}

		todo!("split")
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
