pub trait Helper {
	fn is_digit(&self) -> bool;
	fn is_ident_start(&self) -> bool;
}

impl Helper for &u8 {
	#[inline(always)]
	fn is_digit(&self) -> bool {
		self.is_ascii_digit()
	}

	#[inline(always)]
	fn is_ident_start(&self) -> bool {
		matches!(self, b'a'..=b'z' | b'A'..=b'Z' | b'_')
	}
}

impl Helper for Option<&u8> {
	#[inline(always)]
	fn is_digit(&self) -> bool {
		matches!(self, Some(x) if x.is_digit())
	}

	#[inline(always)]
	fn is_ident_start(&self) -> bool {
		matches!(self, Some(x) if x.is_ident_start())
	}
}
