use super::{Parser, Result, Token};

/// Wrapper around [`Parser`], caches current and prev tokens
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

		let mut tokens = Self {
			parser,
			current,
			prev: None,
		};

		while tokens.current() == Token::Whitespace {
			tokens.next()?;
		}

		Ok(tokens)
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
	pub fn peek_next_with_whitespace(&mut self) -> Result<Token<'a>> {
		self.parser.peek_next()
	}

	#[inline(always)]
	pub fn prev(&self) -> Option<Token<'a>> {
		self.prev
	}
}

impl<'a> std::fmt::Debug for Tokens<'a> {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		if let Some(prev) = self.prev() {
			writeln!(f, "Previous token: {:?}", prev)?;
		}

		writeln!(f, "Current token: {:?}", self.current)?;

		Ok(())
	}
}
