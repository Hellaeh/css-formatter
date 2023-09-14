use super::{Parser, Result, Token};

/// Wrapper around [`Parser`], caches current and prev tokens
#[derive(Debug)]
pub struct Tokens<'a> {
	parser: Parser<'a>,
	current: Token<'a>,
	prev: Option<Token<'a>>,
}

impl<'a> Tokens<'a> {
	#[inline(always)]
	pub fn current(&self) -> Token<'a> {
		self.current
	}

	#[inline]
	pub fn new(mut parser: Parser<'a>) -> Result<Self> {
		let current = parser.next()?;

		Ok(Self {
			parser,
			current,
			prev: None,
		})
	}

	#[inline]
	pub fn next(&mut self) -> Result<Token<'a>> {
		let mut next = self.parser.next()?;

		while next == Token::Whitespace {
			next = self.parser.next()?;
		}

		self.prev = Some(self.current);
		self.current = next;

		Ok(next)
	}

	#[inline]
	pub fn next_with_whitespace(&mut self) -> Result<Token<'a>> {
		let next = self.parser.next()?;

		self.prev = Some(self.current);
		self.current = next;

		Ok(next)
	}

	#[inline(always)]
	pub fn peek_next(&mut self) -> Result<Token<'a>> {
		self.parser.peek_next()
	}

	#[inline(always)]
	pub fn prev(&self) -> Option<Token<'a>> {
		self.prev
	}
}
