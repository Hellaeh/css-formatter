#![feature(specialization)]

// WARNING: Must be power of 2
const SIZE: usize = 16;

/// Fixed length ring buffer
///
/// Proper testing in debug mode required.
/// Unsafe af
pub struct RingBuffer<T, const N: usize = SIZE> {
	buf: [T; N],
	writer: usize,
	reader: usize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
	#[inline]
	pub const fn new(initial: T) -> Self {
		const {
			assert!(
				N.is_power_of_two(),
				"RingBuffer size should be power of two"
			);
		}

		Self {
			buf: [initial; N],
			writer: 1,
			reader: 0,
		}
	}
}

impl<T, const N: usize> RingBuffer<T, N> {
	#[inline]
	pub fn go(&mut self, vector: isize) {
		self.reader = (self.reader as isize + vector) as usize;
	}

	#[inline(always)]
	pub fn go_next(&mut self) {
		self.go(1)
	}

	#[inline(always)]
	pub fn go_prev(&mut self) {
		self.go(-1)
	}

	#[inline]
	pub fn push(&mut self, value: T) {
		debug_assert!(
			self.writer - self.reader < SIZE,
			"`Writer` is lap ahead of `Reader`. This is Undefined Behavior"
		);

		let i = self.writer & (N - 1);

		self.buf[i] = value;
		self.writer += 1;
	}

	#[inline(always)]
	pub fn is_behind(&self) -> bool {
		self.behind() > 0
	}

	#[inline(always)]
	pub fn behind(&self) -> usize {
		self.writer - (self.reader + 1)
	}
}

impl<T, const N: usize> std::ops::Index<isize> for RingBuffer<T, N> {
	type Output = T;

	#[inline]
	fn index(&self, index: isize) -> &Self::Output {
		let i = (self.reader as isize + index) as usize & (N - 1);

		debug_assert!(i < self.writer, "Reading ahead of writer!");

		&self.buf[i]
	}
}

impl<T> std::fmt::Debug for RingBuffer<T> {
	default fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RingBuffer")
			// TODO: Make a repr of an internal buf without exposing T
			// .field("buf", &continious)
			.field("writer", &self.writer)
			.field("reader", &self.reader)
			.finish()
	}
}

impl<T: std::fmt::Debug> std::fmt::Debug for RingBuffer<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RingBuffer")
			.field("buf", &self.buf)
			.field("writer", &self.writer)
			.field("reader", &self.reader)
			.finish()
	}
}
