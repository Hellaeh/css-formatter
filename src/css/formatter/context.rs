use super::{line::Line, utils::Helper};

#[derive(Debug)]
pub struct IntegerOverflow;

#[derive(Debug)]
pub struct Context<T> {
	output: T,
	layers: Vec<Vec<u8>>,

	indent: u8,
	line_num: u32,

	current_line: Line,
	comment: Line,
}

impl<T> Context<T>
where
	T: std::io::Write,
{
	#[inline(always)]
	fn flush_into(
		current_line: &mut Line,
		prev_line: &mut Line,
		line_num: &mut u32,
		indent: u8,
		output: &mut impl std::io::Write,
	) -> std::io::Result<()> {
		if !prev_line.is_empty() {
			*line_num += prev_line.flush_self_with_indent(indent, output)?;
		}

		if current_line.is_empty() {
			*line_num += 1;
			output.write_newline()?;
			output.flush()?;
		} else {
			*line_num += current_line.flush_self_with_indent(indent, output)?;
		}

		Ok(())
	}

	/// Flushes self into [`T`], or current layer if any
	#[inline]
	pub fn flush(&mut self) -> std::io::Result<()> {
		// Ugly AF... oh well
		match self.layers.last_mut() {
			Some(layer) => Self::flush_into(
				&mut self.current_line,
				&mut self.comment,
				&mut self.line_num,
				self.indent,
				layer,
			),
			None => Self::flush_into(
				&mut self.current_line,
				&mut self.comment,
				&mut self.line_num,
				self.indent,
				&mut self.output,
			),
		}
	}

	#[inline]
	pub fn current_output(&mut self) -> &mut dyn std::io::Write {
		if let Some(layer) = self.layers.last_mut() {
			return layer;
		}

		&mut self.output
	}

	#[inline(always)]
	pub fn indent_dec(&mut self) -> Result<(), IntegerOverflow> {
		self.indent = self.indent.checked_sub(1).ok_or(IntegerOverflow)?;
		Ok(())
	}

	#[inline(always)]
	pub fn indent_inc(&mut self) -> Result<(), IntegerOverflow> {
		self.indent = self.indent.checked_add(1).ok_or(IntegerOverflow)?;
		Ok(())
	}

	#[inline(always)]
	pub fn layer_push(&mut self, layer: Vec<u8>) {
		self.layers.push(layer);
	}

	#[inline]
	pub fn layer_take(&mut self) -> std::io::Result<Option<Vec<u8>>> {
		Ok(self.layers.pop())
	}

	#[inline]
	pub fn new(output: T) -> Self {
		Self {
			output,

			layers: Vec::new(),

			indent: 0,
			line_num: 0,

			current_line: Line::new(),
			comment: Line::new(),
		}
	}

	#[inline]
	pub fn take(&mut self) -> Line {
		let mut res = Line::new();

		// let string = String::new();

		if !self.comment.is_empty() {
			res.extend_from_slice(&self.comment);

			unsafe {
				res.write_newline().unwrap_unchecked();
				res.write_indent(self.indent).unwrap_unchecked();
			};

			self.comment.clear();

			self.line_num += 1;
		}

		res.extend_from_slice(&self.current_line);

		self.current_line.clear();

		self.line_num += 1;

		res
	}

	#[inline]
	pub fn write_comment(&mut self, bytes: &[u8]) -> std::io::Result<()> {
		if !self.comment.is_empty() {
			self.comment.write_newline()?;
			self.comment.write_indent(self.indent)?;
		}

		self.comment.write_comment(bytes)?;

		Ok(())
	}

	#[inline]
	pub fn flush_line(&mut self, line: &[u8]) -> std::io::Result<()> {
		let indent = self.indent;
		self.current_output().write_indent(indent)?;
		self.current_output().write_all(line)?;
		self.current_output().write_newline()?;
		self.current_output().flush()
	}
}

impl<T> std::ops::Deref for Context<T> {
	type Target = Line;

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.current_line
	}
}

impl<T> std::ops::DerefMut for Context<T> {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.current_line
	}
}
