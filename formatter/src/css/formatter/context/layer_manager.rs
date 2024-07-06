use crate::css::{formatter::line::Line, properties::Descriptor};

use super::Declaration;

#[derive(Clone, Copy, Default)]
pub struct LayerManager {
	current: usize,
}

#[derive(Default, Debug)]
pub struct DeclarationManager {
	inner: Vec<Declaration>,
	cursor: usize,
}

#[derive(Default)]
pub struct Layer {
	/// Declarations have to be sorted
	declarations: DeclarationManager,
	/// Nested block or at-rules
	main: Line,
}

#[thread_local]
static mut ARENA: Vec<Layer> = Vec::new();

macro_rules! get {
	() => {
		unsafe { &mut *std::ptr::addr_of_mut!(ARENA) }
	};

	($idx: expr) => {
		get!()[$idx]
	};
}

impl LayerManager {
	#[inline]
	pub fn push(&mut self) {
		// We waste zeroth layer
		self.current += 1;

		get!().get_or_init(self.current).clear();
	}

	#[inline]
	pub fn pop(&mut self) -> &'static mut Layer {
		let layer = &mut get![self.current];

		self.current -= 1;

		layer
	}

	#[inline]
	pub fn current(self) -> Option<&'static mut Layer> {
		if self.current == 0 {
			return None;
		}

		Some(&mut get![self.current])
	}
}

impl Layer {
	#[inline]
	fn clear(&mut self) {
		self.main.clear();
		self.declarations.clear();
	}

	#[inline(always)]
	pub fn declarations(&self) -> &DeclarationManager {
		&self.declarations
	}

	#[inline(always)]
	pub fn declarations_mut(&mut self) -> &mut DeclarationManager {
		&mut self.declarations
	}

	#[inline(always)]
	pub fn main(&self) -> &Line {
		&self.main
	}
	#[inline(always)]
	pub fn main_mut(&mut self) -> &mut Line {
		&mut self.main
	}
}

impl DeclarationManager {
	#[inline]
	fn clear(&mut self) {
		self.cursor = 0;
	}

	pub fn push(&mut self, descriptor: Descriptor) -> &mut Line {
		let declaration = self.inner.get_or_init(self.cursor);
		declaration.clear();

		declaration.descriptor = descriptor;

		self.cursor += 1;

		&mut declaration.line
	}

	pub fn pop(&mut self) -> &mut Line {
		debug_assert!(self.cursor > 0, "Popped before push");

		&mut self.inner[self.cursor - 1].line
	}
}

impl<T: Default> Helper for Vec<T> {
	type Output = T;

	#[inline]
	fn get_or_init(&mut self, idx: usize) -> &mut Self::Output {
		while idx >= self.len() {
			self.push(T::default())
		}

		&mut self[idx]
	}
}

trait Helper {
	type Output;

	fn get_or_init(&mut self, idx: usize) -> &mut Self::Output;
}

impl std::ops::Deref for DeclarationManager {
	type Target = [Declaration];

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		&self.inner[..self.cursor]
	}
}

impl std::ops::DerefMut for DeclarationManager {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner[..self.cursor]
	}
}
