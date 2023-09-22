use std::io::Result;

pub trait Helper: std::io::Write {
	#[inline]
	fn write_comment(&mut self, bytes: &[u8]) -> Result<()> {
		self.write_all(b"/* ")?;
		self.write_all(bytes.trim_ascii())?;
		self.write_all(b" */")
	}

	#[inline]
	fn write_indent(&mut self, indent: u8) -> Result<()> {
		for _ in 0..indent {
			self.write_u8(b'\t')?;
		}

		Ok(())
	}

	#[inline(always)]
	fn write_newline(&mut self) -> Result<()> {
		self.write_u8(b'\n')
	}

	#[inline]
	fn finish_line_with_indent(&mut self, bytes: &[u8], indent: u8) -> Result<()> {
		self.write_indent(indent)?;
		self.write_all(bytes)?;
		self.write_newline()
	}

	#[inline(always)]
	fn write_newline_with_indent(&mut self, indent: u8) -> Result<()> {
		self.write_newline()?;
		self.write_indent(indent)
	}

	#[inline(always)]
	fn write_space(&mut self) -> Result<()> {
		self.write_u8(b' ')
	}

	#[inline(always)]
	fn write_u8(&mut self, byte: u8) -> Result<()> {
		self.write_all(&[byte])
	}
}

impl<T: std::io::Write> Helper for T {}
