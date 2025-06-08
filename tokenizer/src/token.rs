// WARNING: KEEP IT COPYABLE
// Actually do not touch it at all
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Token {
	/// Comment token - will not include starting `/*` and ending `*/`
	Comment(*const u8, u32), // Formatter should preserve comments

	Ident(*const u8, u32),
	Function(*const u8, u32),
	// URL(*const u8, u32),
	// TODO: ^
	// BadURL, // Not supported
	AtRule(*const u8, u32),
	Hash(*const u8, u32),
	/// String token - will not include surrounding quotes
	String(*const u8, u32),
	Number(*const u8, u32),

	Delim(u8),
	// Percentage, // Number
	// Dimension, // Number
	/// Whitespace token - any amount of whitespace that matches: [\s*]
	Whitespace(bool),
	Colon,
	Semicolon,
	Comma,
	BracketRoundOpen,
	BracketRoundClose,
	BracketSquareOpen,
	BracketSquareClose,
	BracketCurlyOpen,
	BracketCurlyClose,
}

const _ASSERT_SIZE: () = {
	const SIZE_LIMIT: usize = 16;
	macro_rules! m {
		() => {
			"Token struct size exceeds 16 bytes"
		};
	}
	use std::mem::size_of;
	assert!(size_of::<Token>() == SIZE_LIMIT, m!());
	assert!(size_of::<Option<Token>>() == SIZE_LIMIT, m!());
	assert!(size_of::<Result<Option<Token>, ()>>() == SIZE_LIMIT, m!());
	assert!(size_of::<crate::Result>() == SIZE_LIMIT, m!());
};

const _ASSERT_TRANSMUTE: () = {
	const LENGTH: usize = 10;
	let token = Token::Ident(std::ptr::null(), LENGTH as u32);
	let bytes = token.bytes();
	assert!((bytes as *const u8).is_null());
	assert!(bytes.len() == LENGTH);
};

macro_rules! bytes_pat {
	($($des: tt)*) => {
		| Token::Comment($($des)*)
		| Token::Ident($($des)*)
		| Token::Function($($des)*)
		| Token::AtRule($($des)*)
		| Token::Hash($($des)*)
		| Token::String($($des)*)
		| Token::Number($($des)*)
	};
}

impl Token {
	#[inline]
	pub const fn bytes(self) -> *const [u8] {
		debug_assert!(matches!(self, bytes_pat!(..)));

		// HACK: Dangerous AF - tests should cover it
		let (_, len, ptr) = unsafe { std::mem::transmute::<Token, (u32, u32, *const u8)>(self) };
		std::ptr::slice_from_raw_parts(ptr, len as usize)
	}

	#[inline(always)]
	pub fn from<F>(variant: F, ptr: *const u8, end: *const u8) -> Self
	where
		F: Fn(*const u8, u32) -> Self,
	{
		variant(ptr, (end.addr() - ptr.addr()) as u32)
	}
}

impl std::fmt::Display for Token {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		use std::str::from_utf8_unchecked as str;

		#[inline(always)]
		fn fmt_newline(newline: bool) -> &'static str {
			if newline {
				"Whitespace with linebreak"
			} else {
				"Whitespace"
			}
		}

		unsafe {
			#[allow(unreachable_patterns)]
			match self {
				Token::Comment(..) => write!(f, "Comment(\"{}\")", str(&*self.bytes())),
				Token::Ident(..) => write!(f, "Ident(\"{}\")", str(&*self.bytes())),
				Token::Function(..) => write!(f, "Function(\"{}\")", str(&*self.bytes())),
				Token::AtRule(..) => write!(f, "AtRule(\"{}\")", str(&*self.bytes())),
				Token::Hash(..) => write!(f, "Hash(\"{}\")", str(&*self.bytes())),
				Token::String(..) => write!(f, "String(\"{}\")", str(&*self.bytes())),
				Token::Number(..) => write!(f, "Number(\"{}\")", str(&*self.bytes())),

				Token::Delim(d) => write!(f, "Delim({})", *d as char),

				Token::Whitespace(newline) => f.write_str(fmt_newline(*newline)),
				Token::Colon => f.write_str("Colon"),
				Token::Semicolon => f.write_str("Semicolon"),
				Token::Comma => f.write_str("Comma"),
				Token::BracketRoundOpen => f.write_str("BracketRoundOpen"),
				Token::BracketRoundClose => f.write_str("BracketRoundClose"),
				Token::BracketSquareOpen => f.write_str("BracketSquareOpen"),
				Token::BracketSquareClose => f.write_str("BracketSquareClose"),
				Token::BracketCurlyOpen => f.write_str("BracketCurlyOpen"),
				Token::BracketCurlyClose => f.write_str("BracketCurlyClose"),

				_ => todo!(),
			}
		}
	}
}
