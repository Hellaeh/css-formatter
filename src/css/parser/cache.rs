use super::{Parser, Result, Token};

use self::ring_buffer::RingBuffer;

/// Wrapper around [`Parser`], caches current and prev tokens
/// also allow peeking
pub struct Cache<'a> {
	parser: Parser<'a>,

	/// Will contain a queue of tokens, with 0 index represent
	/// current token
	ring: RingBuffer<Option<Token<'a>>>,
}

impl<'a> Cache<'a> {
	#[inline(always)]
	pub fn current(&self) -> Token<'a> {
		unsafe { self.ring[0].unwrap_unchecked() }
	}

	#[inline(always)]
	pub fn get(&self, pos: isize) -> Option<&Token<'a>> {
		debug_assert!((-8..=7).contains(&pos));

		self.ring[pos].as_ref()
	}

	#[inline]
	pub fn prev(&mut self) -> Option<Token<'a>> {
		match self.prev_with_whitespace() {
			Some(Token::Whitespace) => self.prev_with_whitespace(),
			token => token,
		}
	}

	#[inline]
	pub fn prev_with_whitespace(&mut self) -> Option<Token<'a>> {
		let prev = self.ring[-1];

		if prev.is_some() {
			self.ring.go_prev()
		}

		prev
	}

	#[inline]
	pub fn new(mut parser: Parser<'a>) -> Result<Self> {
		let mut token = parser.next()?;

		if token == Token::Whitespace {
			token = parser.next()?;
		}

		let ring = RingBuffer::new(Some(token));

		Ok(Self { parser, ring })
	}

	#[inline]
	pub fn next(&mut self) -> Result<Token<'a>> {
		let mut next = self.next_with_whitespace()?;

		if next == Token::Whitespace {
			next = self.next_with_whitespace()?;
		}

		Ok(next)
	}

	#[inline]
	pub fn next_with_whitespace(&mut self) -> Result<Token<'a>> {
		let next = self.peek_next_with_whitespace()?;
		self.ring.go_next();
		Ok(next)
	}

	#[inline]
	pub fn peek(&mut self, pos: usize) -> Result<Token<'a>> {
		debug_assert!(pos < 8);

		while self.ring.behind() < pos {
			let next = self.parser.next()?;
			self.ring.push(Some(next))
		}

		Ok(self[pos as isize])
	}

	/// Will try to peek a next token that is not [`Token::Whitespace`]
	#[inline]
	pub fn peek_next(&mut self) -> Result<Token<'a>> {
		match self.peek(1) {
			Ok(Token::Whitespace) => self.peek(2),
			token => token,
		}
	}

	#[inline(always)]
	pub fn peek_next_with_whitespace(&mut self) -> Result<Token<'a>> {
		self.peek(1)
	}

	#[inline]
	pub fn peek_prev(&self) -> Option<Token<'a>> {
		match self.ring[-1] {
			Some(Token::Whitespace) => self.ring[-2],
			other => other,
		}
	}

	#[inline(always)]
	pub fn peek_prev_with_whitespace(&self) -> Option<Token<'a>> {
		self.ring[-1]
	}
}

impl<'a> std::ops::Index<isize> for Cache<'a> {
	type Output = Token<'a>;

	#[inline(always)]
	fn index(&self, index: isize) -> &Self::Output {
		unsafe { self.get(index).as_ref().unwrap_unchecked() }
	}
}

impl<'a> std::fmt::Debug for Cache<'a> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(prev) = self.peek_prev_with_whitespace() {
			writeln!(f, "Previous token: {:?}", prev)?;
		}

		write!(f, "Current token: {:?}", self.current())?;

		Ok(())
	}
}

mod ring_buffer;
