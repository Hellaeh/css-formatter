use super::{context::Context, Helper};

const MAX_LENGTH: usize = 100;

#[derive(Debug)]
pub struct LineWithContext<'a> {
	line: Line,
	context: &'a Context,
}

#[derive(Debug, Clone)]
pub struct Line {
	buf: Vec<u8>,
}

impl<'a> LineWithContext<'a> {
	#[inline]
	pub fn new(context: &'a Context) -> Self {
		Self {
			line: Line::new(),
			context,
		}
	}

	/// Will clone and return cloned [`Line`], while clearing original buffer.
	/// Reason - making it cache friendly by keeping original buffer pointer in place
	#[inline]
	pub fn end(&mut self) -> Line {
		let res = self.line.clone();

		self.line.buf.clear();

		res
	}

	/// Will take underling [`Line`], while consuming self
	#[inline]
	pub fn take(self) -> Line {
		self.line
	}

	#[inline(always)]
	pub fn write_indent(&mut self) -> std::io::Result<()> {
		self.line.write_indent(self.context.indentation())
	}
}

#[allow(unused_must_use)]
impl Line {
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

impl<'a> std::ops::Deref for LineWithContext<'a> {
	type Target = Line;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.line
	}
}

impl<'a> std::ops::DerefMut for LineWithContext<'a> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.line
	}
}

impl std::io::Write for Line {
	#[inline(always)]
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		self.buf.write(buf)
	}

	#[inline(always)]
	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl<'a> std::io::Write for LineWithContext<'a> {
	#[inline(always)]
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		if self.is_empty() {
			self.write_indent()?;
		}

		Vec::write(&mut self.buf, buf)
	}

	#[inline(always)]
	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

impl Ord for Line {
	#[inline(always)]
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.buf.cmp(&other.buf)
	}
}

impl PartialOrd for Line {
	#[inline(always)]
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		Some(self.cmp(other))
	}
}

impl Eq for Line {}
impl PartialEq for Line {
	#[inline(always)]
	fn eq(&self, other: &Self) -> bool {
		self.buf == other.buf
	}
}
