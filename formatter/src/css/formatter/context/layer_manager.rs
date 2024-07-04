use crate::css::formatter::{line::Line, Declaration};

pub struct LayerManager;

pub trait Helper {
	type Output;

	fn get_or_init(&mut self, idx: usize) -> Self::Output;
}

#[thread_local]
static mut DECLARATION_ARENA: Vec<Vec<Declaration>> = Vec::new();
#[thread_local]
static mut BLOCK_ARENA: Vec<Vec<Line>> = Vec::new();

macro_rules! get {
	($arena: ident) => {
		unsafe { &mut *std::ptr::addr_of_mut!($arena) }
	};
}

impl LayerManager {
	#[inline]
	pub fn get_declarations(layer: u8) -> Vec<Declaration> {
		let arena = get!(DECLARATION_ARENA);

		arena.get_or_init(layer as usize)
	}

	#[inline]
	pub fn set_declarations(layer: u8, declarations: Vec<Declaration>) {
		let arena = get!(DECLARATION_ARENA);

		arena[layer as usize] = declarations;
	}
}

impl<T: Default> Helper for Vec<T> {
	type Output = T;

	#[inline]
	fn get_or_init(&mut self, idx: usize) -> Self::Output {
		while idx >= self.len() {
			self.push(T::default())
		}

		std::mem::take(&mut self[idx])
	}
}
