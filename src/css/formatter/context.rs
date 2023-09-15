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
	queue: Vec<Line>,
}

impl<T> Context<T>
where
	T: std::io::Write,
{
	#[inline(always)]
	fn flush_into(
		current_line: &mut Line,
		queue: &mut Vec<Line>,
		line_num: &mut u32,
		indent: u8,
		output: &mut impl std::io::Write,
	) -> std::io::Result<()> {
		while let Some(mut line) = queue.pop() {
			*line_num += line.write_with_indent_into(indent, output)?;
		}

		// if !current_line.is_empty() {
		*line_num += current_line.write_with_indent_into(indent, output)?;
		current_line.clear();
		// }

		Ok(())
	}

	/// Flushes self into [`T`], or current layer if any
	#[inline]
	pub fn flush(&mut self) -> std::io::Result<()> {
		// Ugly AF... oh well
		match self.layers.last_mut() {
			Some(layer) => Self::flush_into(
				&mut self.current_line,
				&mut self.queue,
				&mut self.line_num,
				self.indent,
				layer,
			),
			None => Self::flush_into(
				&mut self.current_line,
				&mut self.queue,
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
			queue: Vec::new(),
		}
	}

	#[inline(always)]
	pub fn write_newline(&mut self) -> std::io::Result<()> {
		self.line_num += 1;

		self.current_line.write_newline()?;
		self.flush()?;

		Ok(())
	}

	#[inline]
	pub fn take(&mut self) -> Line {
		let res = self.current_line.clone();

		self.line_num += 1;
		self.current_line.clear();

		res
	}

	#[inline]
	pub fn write_comment(&mut self, bytes: &[u8]) -> std::io::Result<()> {
		let mut line = Line::new();
		line.write_comment(bytes)?;
		self.queue.push(line);

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
