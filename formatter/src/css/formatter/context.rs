use super::{line::Line, utils::Helper};

pub use layer_manager::Helper as LayerHelper;
pub use layer_manager::LayerManager;

#[derive(Debug)]
pub struct IntegerOverflow;

pub struct Context<T> {
	output: T,
	layers: Vec<Vec<u8>>,

	indent: u8,
	line_num: u32,

	current_line: Line,
}

impl<T> Context<T>
where
	T: std::io::Write,
{
	#[inline]
	fn flush_into(
		current_line: &mut Line,
		line_num: &mut u32,
		indent: u8,
		output: &mut impl std::io::Write,
	) -> std::io::Result<()> {
		if current_line.is_empty() {
			*line_num += 1;
			output.write_newline()?;
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
				&mut self.line_num,
				self.indent,
				layer,
			),
			None => Self::flush_into(
				&mut self.current_line,
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

	#[inline]
	pub fn indent_dec(&mut self) -> Result<(), IntegerOverflow> {
		self.indent = self.indent.checked_sub(1).ok_or(IntegerOverflow)?;
		Ok(())
	}

	#[inline]
	pub fn indent_inc(&mut self) -> Result<(), IntegerOverflow> {
		self.indent = self.indent.checked_add(1).ok_or(IntegerOverflow)?;
		Ok(())
	}

	#[inline]
	pub fn layer_push(&mut self, layer: Vec<u8>) {
		self.layers.push(layer);
	}

	#[inline]
	pub fn layer_pop(&mut self) -> Option<Vec<u8>> {
		self.layers.pop()
	}

	#[inline]
	pub fn replace_line(&mut self, line: Line) -> Line {
		std::mem::replace(&mut self.current_line, line)
	}

	#[inline]
	pub fn new(output: T) -> Self {
		Self {
			output,

			layers: Vec::new(),

			indent: 0,
			line_num: 0,

			current_line: Line::new(),
		}
	}

	#[inline]
	pub fn take(&mut self) -> Line {
		let res = self.current_line.clone();

		self.current_line.clear();

		self.line_num += 1;

		res
	}

	#[inline(always)]
	pub fn indent(&self) -> u8 {
		self.indent
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

impl<T> std::fmt::Debug for Context<T> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let buf = unsafe { std::str::from_utf8_unchecked(&self.current_line) };

		writeln!(f, "Line: {}", self.line_num)?;
		writeln!(f, "Indentation: {}", self.indent)?;
		write!(f, "Content: \"{}\"", buf)
	}
}

mod layer_manager;
