use std::{assert_matches::debug_assert_matches, fmt::Write, hint::unreachable_unchecked};

use consts::ASCII;

#[derive(Clone, Copy, Debug)]
#[non_exhaustive]
pub enum Kind {
	OpenGroupStart,
	ClosedGroupStart,
	ClosedGroupEnd,
	Point,

	/// Depends on previous node offset
	Conditional,

	Operator,
	Whitespace,

	// Plugs
	Begin,
	End,
	None,
}

#[derive(Clone, Copy)]
pub struct SplitNode {
	/// Self id
	pub id: u32,

	/// At location
	pub at: u32,
	pub kind: Kind,
	/// Associated metadata - depends on `Kind`
	/// `Kind::ClosedGroupStart` => "end_id" - id of a node with `Kind::ClosedGroupEnd`
	pub metadata: u32,

	pub offset_indent: u8,
	pub offset_from: i8,
	pub offset_to: i8,
}

#[derive(Clone, Copy)]
pub struct SplitTree;

#[thread_local]
static mut ARENA: Vec<SplitNode> = Vec::new();

#[inline]
fn unsafe_get_arena() -> &'static mut Vec<SplitNode> {
	unsafe { &mut *std::ptr::addr_of_mut!(ARENA) }
}

impl SplitTree {
	#[inline]
	pub fn from_byte_array(buf: &[u8]) -> Self {
		let splits = unsafe_get_arena();
		let mut split;
		let mut window;
		let mut offset = 0;
		let mut at = 0;
		let mut open_group = false;

		/// Helper macro to construct `SplitNode`
		macro_rules! node {
		 	($kind: expr, $from: expr, $to: expr$(, $key: ident: $value: expr$(,)?)*) => {
		 		node!(kind: $kind, offset_from: $from, offset_to: $to, $($key: $value,)*)
		 	};

		 	($kind: expr$(, $key: ident: $value: expr$(,)?)*) => {
		 		node!(kind: $kind, $($key: $value,)*)
		 	};

		 	($($key: ident: $value: expr$(,)?)*) => {
		 		SplitNode {
		 			at: at as u32,

		 			offset_indent: offset,
		 			$($key: $value,)*
		 			..Default::default()
		 		}
		 	};
		 }

		// truncate(1) - looks slower, testing required
		splits.clear();
		splits.push(node!(Kind::Begin));

		let mut windows = buf.array_windows::<2>().enumerate();

		loop {
			(at, window) = match windows.next() {
				Some(next) => next,
				None => break,
			};

			match window {
				// Matches `content: "..."` or `[href^="#"]`
				//                   ^                 ^
				[ASCII::DOUBLE_QUOTE, ..] => {
					loop {
						match window {
							// Skip next escaped quote
							[ASCII::BACKSLASH, ASCII::DOUBLE_QUOTE] => {}
							[.., ASCII::DOUBLE_QUOTE] => break, // at as u32 - split.at + 1,
							_ => {}
						};

						window = unsafe { windows.next().unwrap_unchecked().1 };
					}

					windows.next();

					continue;
				}

				// Matches `rgb(...)` or `[class="..."]`
				//             ^          ^
				[ASCII::PAREN_OPEN | ASCII::SQUARED_OPEN, ..] => {
					offset += 1;

					split = node!(Kind::ClosedGroupStart, 1, 1);
				}

				// Matches `rgb(...)` or `[class="..."]`
				//                 ^                  ^
				[ASCII::PAREN_CLOSE | ASCII::SQUARED_CLOSE, ..] => {
					offset -= 1;

					split = node!(Kind::ClosedGroupEnd);
				}

				// Matches `background: ...`, but not `div:hover` or `:has(...`
				//                    ^                   x           x
				[ASCII::COLON, ASCII::SPACE] => {
					offset += 1;
					open_group = true;

					split = node!(Kind::OpenGroupStart, 2, 1);

					// Skip space
					windows.next();
				}

				// Matches `blue, red`, but not `div,` followed by line ending
				//              ^                   X
				[ASCII::COMMA, ASCII::SPACE] => {
					split = node!(Kind::Point, 2, 1);

					// Skip space
					windows.next();
				}

				// Matches `.class > a`
				//                 ^
				[ASCII::SPACE, ASCII::GT | ASCII::PLUS | ASCII::TILDE] => {
					// Skip space
					at = unsafe { windows.next().unwrap_unchecked().0 };
					offset += 1;

					split = node!(Kind::Operator, 0, -1);

					// Skip delim
					windows.next();
					offset -= 1;
				}

				[ASCII::SPACE, ASCII::CURLY_OPEN] => {
					split = node!(Kind::Conditional, 1, 0);
				}

				// Matches `.class a`, but not `.class > a` (above rule) or `red, blue`
				//                ^                   x x                        x
				[ASCII::SPACE, ..] => {
					offset += !open_group as u8;

					split = node!(Kind::Whitespace, 1, 0);

					offset -= !open_group as u8;
				}

				_ => continue,
			}

			splits.push(split);
		}

		// Check last char
		at = buf.len() - 1;
		if matches!(buf[at], ASCII::PAREN_CLOSE | ASCII::SQUARED_CLOSE) {
			splits.push(node!(Kind::ClosedGroupEnd));
		}

		at = buf.len();
		splits.push(node!(Kind::End));

		#[cfg(debug_assertions)]
		{
			let mut out = String::with_capacity(buf.len() * 2 + 10);

			out.push_str("Found splits for:\n");

			out.push('`');
			out.push_str(unsafe { std::str::from_utf8_unchecked(buf) });
			out.push('`');
			out.push('\n');

			out.push(' ');

			let mut prev = 0;
			// NOTE: Skip 1st and last elements
			for node in splits.iter().skip(1).take(unsafe_get_arena().len() - 2) {
				let at = node.at;

				for _ in prev..at {
					out.push(' ');
				}

				out.push('^');

				prev = at + 1;
			}

			eprintln!("{out}\n");
		}

		Self.update()
	}

	#[inline]
	fn update(self) -> Self {
		#[inline]
		fn inner<'a>(last: usize, iter: &mut impl Iterator<Item = &'a mut SplitNode>) -> u32 {
			while let Some(node) = iter.next() {
				match &mut node.kind {
					Kind::ClosedGroupStart => node.metadata = inner(last, iter),
					Kind::ClosedGroupEnd => return node.id,

					_ => {}
				}
			}

			last as u32
		}

		inner(self.len() - 1, &mut self.iter_mut());

		self
	}

	#[inline]
	pub fn into_iter(self) -> impl Iterator<Item = SplitNode> {
		unsafe_get_arena()
			.iter()
			.copied()
			.filter(|s| !matches!(s.kind, Kind::None))
	}

	#[inline]
	pub fn iter(&self) -> impl DoubleEndedIterator<Item = &SplitNode> {
		unsafe_get_arena()
			.iter()
			.filter(|s| !matches!(s.kind, Kind::None))
	}

	#[inline]
	pub fn iter_mut(&self) -> impl DoubleEndedIterator<Item = &mut SplitNode> {
		unsafe_get_arena()
			.iter_mut()
			.filter(|s| !matches!(s.kind, Kind::None))
	}
}

impl SplitNode {
	#[inline]
	pub fn remove(&mut self) {
		self.kind = Kind::None;
	}

	#[inline]
	pub unsafe fn collapse_closed_group(&mut self) {
		debug_assert_matches!(self.kind, Kind::ClosedGroupStart);

		let Kind::ClosedGroupStart = self.kind else {
			unsafe { unreachable_unchecked() }
		};

		let vec = unsafe_get_arena();

		for i in self.id..=self.metadata {
			vec[i as usize].kind = Kind::None;
		}
	}

	#[inline]
	pub fn next(&self) -> Option<&mut SplitNode> {
		unsafe_get_arena()
			.iter_mut()
			.skip(self.id as usize + 1)
			.find(|s| !s.is_plug())
	}

	#[inline]
	pub fn next_solid(&self) -> Option<&mut SplitNode> {
		unsafe_get_arena()
			.iter_mut()
			.skip(self.id as usize + 1)
			.find(|s| s.is_solid())
	}

	#[inline]
	pub fn prev(&self) -> Option<&mut SplitNode> {
		unsafe_get_arena()
			.iter_mut()
			.take(self.id as usize)
			.rev()
			.find(|s| !s.is_plug())
	}

	#[inline]
	pub fn prev_solid(&self) -> Option<&mut SplitNode> {
		unsafe_get_arena()
			.iter_mut()
			.take(self.id as usize)
			.rev()
			.find(|s| s.is_solid())
	}

	#[inline]
	pub unsafe fn calc_group_len(&self) -> usize {
		debug_assert_matches!(self.kind, Kind::ClosedGroupStart);

		let Kind::ClosedGroupStart = self.kind else {
			unsafe { unreachable_unchecked() }
		};

		(unsafe_get_arena()[self.metadata as usize].at - self.at - 1) as usize
	}

	#[inline]
	pub fn is_solid(&self) -> bool {
		matches!(
			self.kind,
			Kind::ClosedGroupStart | Kind::OpenGroupStart | Kind::Point
		)
	}

	#[inline]
	pub fn is_soft(&self) -> bool {
		matches!(self.kind, Kind::Operator | Kind::Whitespace)
	}

	#[inline]
	pub fn is_plug(&self) -> bool {
		matches!(self.kind, Kind::Begin | Kind::End | Kind::None)
	}
}

impl std::fmt::Debug for SplitTree {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut type_name = std::any::type_name::<SplitTree>();
		if let Some(id) = type_name.rfind(&[':', ':']) {
			type_name = type_name.split_at(id + 1).1;
		};

		write!(f, "{} {{", type_name)?;

		let mut wrote = false;

		for node in self.iter() {
			if node.is_plug() {
				continue;
			}

			f.write_char('\n')?;

			let offset = match node.kind {
				Kind::OpenGroupStart | Kind::ClosedGroupStart => node.offset_indent - 1,
				_ => node.offset_indent,
			};

			for _ in 0..=offset {
				f.write_str("    ")?;
			}

			write!(f, "{node:?}")?;

			wrote = true;
		}

		if wrote {
			f.write_char('\n')?;
		}

		f.write_char('}')
	}
}

impl std::fmt::Debug for SplitNode {
	#[inline]
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let mut type_name = std::any::type_name::<SplitNode>();
		if let Some(id) = type_name.rfind(&[':', ':']) {
			type_name = type_name.split_at(id + 1).1;
		};

		macro_rules! helper {
			($field: ident) => {
				if self.$field != 0 {
					write!(f, ", {}: {}", stringify!($field), self.$field)?
				}
			};
		}

		write!(f, "{} {{ ", type_name)?;

		write!(f, "at: {}, ", self.at)?;
		write!(f, "kind: {:?}", self.kind)?;

		if let Kind::ClosedGroupStart = self.kind {
			write!(f, ", metadata(end_id): {}", self.metadata)?
		};

		helper!(offset_from);
		helper!(offset_to);

		write!(f, " }}")
	}
}

impl std::ops::Deref for SplitTree {
	type Target = [SplitNode];

	#[inline(always)]
	fn deref(&self) -> &Self::Target {
		unsafe_get_arena()
	}
}

impl std::ops::DerefMut for SplitTree {
	#[inline(always)]
	fn deref_mut(&mut self) -> &mut Self::Target {
		unsafe_get_arena()
	}
}

impl Default for SplitNode {
	#[inline]
	fn default() -> Self {
		Self {
			id: unsafe_get_arena().len() as u32,

			at: 0,
			kind: Kind::None,
			metadata: 0,

			offset_indent: 0,
			offset_from: 0,
			offset_to: 0,
		}
	}
}

impl Default for Kind {
	#[inline]
	fn default() -> Self {
		Self::None
	}
}
