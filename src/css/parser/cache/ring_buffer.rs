// WARNING: Must be power of 2
const SIZE: usize = 1 << 4;
const BITS: usize = SIZE - 1;

/// Fixed length ring buffer
///
/// Proper testing in debug mode required.
/// Generally unsafe af
pub struct RingBuffer<T, const N: usize = SIZE> {
	buf: [T; N],
	writer: usize,
	reader: usize,
}

impl<T: Copy, const N: usize> RingBuffer<T, N> {
	#[inline]
	pub fn new(initial: T) -> Self {
		debug_assert!(N.is_power_of_two());

		Self {
			buf: [initial; N],
			writer: 1,
			reader: 0,
		}
	}
}

impl<T> RingBuffer<T> {
	#[inline(always)]
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

		let i = self.writer & BITS;

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

impl<T> std::ops::Index<isize> for RingBuffer<T> {
	type Output = T;

	#[inline(always)]
	fn index(&self, index: isize) -> &Self::Output {
		let i = (self.reader as isize + index) as usize & BITS;

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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn basic() {
		let mut ring = RingBuffer::new(0);
		ring.push(42);

		assert_eq!(ring[0], 0);
		assert_eq!(ring[1], 42);

		ring.go(2);

		// Write full circle
		for i in 0..SIZE - 1 {
			ring.push((i + 1) * 10);
		}

		assert!(ring.is_behind());

		assert_eq!(ring[0], 10);

		while ring.is_behind() {
			ring.go(1);
		}

		assert_eq!(ring[0], (14 + 1) * 10);
	}
}
