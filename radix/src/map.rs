use crate::Inner;

/// ASCII only - [a-zA-Z0-9] + '-' + '_'
const CSS_IDENTS_CHARS: usize = 26 * 2 + 10 + 2;

const U8_TO_IDX: [usize; 128] = {
	let mut map = [0; 128];
	let mut i = 0;

	macro_rules! helper {
		($start: expr, $end: expr) => {
			let mut c = $start as usize;

			while c <= $end as usize {
				map[c] = i;

				c += 1;
				i += 1;
			}
		};
	}

	helper!(b'0', b'9');
	helper!(b'a', b'z');
	helper!(b'A', b'Z');

	map[b'-' as usize] = i;
	map[b'_' as usize] = i + 1;

	map
};

pub struct Map<T> {
	inner: [Inner<T>; CSS_IDENTS_CHARS],
}

impl<T> Map<T> {
	#[inline]
	pub fn new() -> Self {
		Self {
			inner: [const { Inner::None }; 64],
		}
	}

	#[inline]
	pub fn get(&self, key: u8) -> &Inner<T> {
		&self.inner[U8_TO_IDX[key as usize]]
	}

	#[inline]
	pub fn get_mut(&mut self, key: u8) -> &mut Inner<T> {
		&mut self.inner[U8_TO_IDX[key as usize]]
	}
}

impl<T> std::fmt::Debug for Map<T>
where
	T: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Map")
			.field_with("inner", |f| {
				f.debug_list()
					.entries(
						self
							.inner
							.iter()
							.enumerate()
							.flat_map(|(i, inner)| match inner {
								Inner::None => None,
								rest => Some(rest),
							}),
					)
					.finish()
			})
			.finish()
	}
}
