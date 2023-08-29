use std::cell::UnsafeCell;

#[derive(Debug)]
pub struct Indentation(UnsafeCell<u8>);

#[derive(Debug)]
pub struct Context {
	indent: Indentation,
	//
	line_num: u32,
}

#[derive(Debug)]
pub struct IntegerUnderflow;

impl Indentation {
	#[inline]
	pub fn new() -> Self {
		Self(UnsafeCell::new(0))
	}

	#[inline(always)]
	pub fn inc(&self) {
		unsafe {
			*self.0.get() += 1;
		}
	}

	#[inline(always)]
	pub fn dec(&self) -> Result<(), IntegerUnderflow> {
		unsafe {
			*self.0.get() = (*self.0.get()).checked_sub(1).ok_or(IntegerUnderflow)?;
		}

		Ok(())
	}
}

impl Context {
	#[inline]
	pub fn new() -> Self {
		Self {
			indent: Indentation::new(),
			line_num: 0,
		}
	}

	#[inline(always)]
	pub fn indentation(&self) -> &Indentation {
		&self.indent
	}

	// pub fn
}

impl std::ops::Deref for Indentation {
	type Target = u8;

	fn deref(&self) -> &Self::Target {
		unsafe { &*self.0.get() }
	}
}
