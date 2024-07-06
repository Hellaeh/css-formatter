use std::io::Write;

use crate::css::properties::{Descriptor, Trie};

use super::{line::Line, utils::Helper};

use consts::ASCII;
use layer_manager::LayerManager;

#[derive(Debug)]
pub struct IntegerOverflow;

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Declaration {
	pub descriptor: Descriptor,
	pub line: Line,
}

pub struct Context<T> {
	output: T,
	layers: LayerManager,

	indent: u8,
	line_num: u32,

	current_line: Line,

	props: Trie,
}

impl<T> Context<T>
where
	T: std::io::Write,
{
	#[inline]
	pub fn declaration_end(&mut self) {
		let current_layer = unsafe { self.layers.current().unwrap_unchecked() };
		let declarations = current_layer.declarations_mut();

		let line = declarations.pop();

		std::mem::swap(&mut self.current_line, line);
	}

	#[inline]
	pub fn declaration_start(&mut self, with: &[u8]) {
		let current_layer = unsafe { self.layers.current().unwrap_unchecked() };
		let declarations = current_layer.declarations_mut();

		let desc = self.get_descriptor(with);
		let line = declarations.push(desc);

		std::mem::swap(&mut self.current_line, line);
	}

	/// Flushes self into [`T`], or current layer if any
	#[inline]
	pub fn flush(&mut self) -> std::io::Result<()> {
		match self.layers.current() {
			Some(layer) => Self::flush_into(
				&mut self.current_line,
				&mut self.line_num,
				self.indent,
				layer.main_mut(),
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

	#[inline]
	fn get_descriptor(&self, bytes: &[u8]) -> Descriptor {
		let name = unsafe { std::str::from_utf8_unchecked(bytes) };

		if bytes.len() > 1 && bytes[0] == ASCII::DASH {
			return if bytes[1] == ASCII::DASH {
				// --variable: somevalue
				Descriptor::variable(name)
			} else {
				// -webkit-line-clamp
				Descriptor::unknown(name)
			};
		}

		if let Some(desc) = self.props.get(bytes) {
			return *desc;
		}

		Descriptor::unknown(name)
	}

	#[inline(always)]
	pub fn indent(&self) -> u8 {
		self.indent
	}

	#[inline]
	pub fn indent_dec(&mut self) -> Result<u8, IntegerOverflow> {
		self.indent = self.indent.checked_sub(1).ok_or(IntegerOverflow)?;
		Ok(self.indent)
	}

	#[inline]
	pub fn indent_inc(&mut self) -> Result<u8, IntegerOverflow> {
		self.indent = self.indent.checked_add(1).ok_or(IntegerOverflow)?;
		Ok(self.indent)
	}

	#[inline]
	pub fn layer_pop(&mut self) -> std::io::Result<()> {
		debug_assert!(
			self.indent > 0,
			"Logical error - popped a layer before any were pushed"
		);

		debug_assert!(
			self.current_line.is_empty(),
			"Current buffer should be empty at this point!"
		);

		let layer = self.layers.pop();

		let declarations: &mut [Declaration] = layer.declarations_mut();

		if !declarations.is_empty() {
			declarations.sort();

			let mut group = unsafe { declarations.first().unwrap_unchecked() }
				.descriptor
				.group();

			for Declaration { descriptor, line } in declarations.iter() {
				if descriptor.group() != group {
					group = descriptor.group();
					self.flush()?;
				}

				self.write_all(line)?;
				self.flush()?;
			}

			if !layer.main().is_empty() {
				self.flush()?;
			}
		}

		// WARNING: DO NOT REORDER
		unsafe { self.indent_dec().unwrap_unchecked() };

		{
			// This is already formatted
			let main = layer.main_mut();

			// ... so we just flush it down the stack
			match self.layers.current() {
				Some(layer) => layer.main_mut().write_all(main),
				None => self.output.write_all(main),
			}?;
		}

		Ok(())
	}

	#[inline]
	pub fn layer_push(&mut self) -> Result<(), IntegerOverflow> {
		self.indent_inc()?;
		self.layers.push();
		Ok(())
	}

	#[inline]
	pub fn new(output: T) -> Self {
		Self {
			output,

			layers: LayerManager::default(),

			indent: 0,
			line_num: 0,

			current_line: Line::new(),

			props: Trie::new(),
		}
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

impl Declaration {
	#[inline]
	fn clear(&mut self) {
		self.line.clear()
	}
}

mod layer_manager;
