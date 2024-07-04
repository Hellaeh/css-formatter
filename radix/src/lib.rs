#![feature(test)]
#![feature(variant_count)]
#![feature(debug_closure_helpers)]

use std::hint::unreachable_unchecked;

use map::Map;

#[derive(Debug)]
enum Inner<T> {
	None,
	Value(T),
	Node(Box<Node<T>>),
	NodeWithValue((T, Box<Node<T>>)),
}

enum Node<T> {
	Empty,
	KeyInner(*const [u8], Inner<T>),
	Map(Map<T>),
}

#[derive(Debug)]
#[non_exhaustive]
enum Equality {
	SharePrefixUpTo(usize),
	Full,
	None,
	Shorter,
	Longer,
}

#[derive(Debug)]
pub struct Radix<T> {
	root: Node<T>,
}

trait Compare {
	fn compare(self, other: Self) -> Equality;
}

impl<T> Node<T> {
	#[inline]
	fn get(&self, key: &[u8]) -> Option<&T> {
		debug_assert!(!key.is_empty());

		let (inner, rest) = match *self {
			Node::Empty => return None,

			Node::KeyInner(current_key, ref inner) => {
				let current_key = unsafe { &*current_key };
				let (key, rest) = key.split_at_checked(current_key.len())?;

				if current_key != key {
					return None;
				}

				(inner, rest)
			}

			Node::Map(ref map) => {
				let (&key, rest) = unsafe { key.split_first().unwrap_unchecked() };
				let inner = map.get(key);

				(inner, rest)
			}
		};

		if rest.is_empty() {
			let (Inner::Value(v) | Inner::NodeWithValue((v, _))) = inner else {
				return None;
			};

			Some(v)
		} else {
			let (Inner::Node(node) | Inner::NodeWithValue((_, node))) = inner else {
				return None;
			};

			node.get(rest)
		}
	}

	fn insert(&mut self, key: &[u8], value: T) {
		assert!(!key.is_empty());

		match *self {
			Node::Empty => *self = Node::KeyInner(key, Inner::Value(value)),

			Node::KeyInner(current_key, ref mut inner) => {
				match current_key.compare(key) {
					// Split self into two nodes `Node::Map` => `Node::Seq`
					Equality::None => {
						self.replace_with(|old| {
							let Node::KeyInner(old_key, old_inner) = old else {
								unsafe { unreachable_unchecked() };
							};

							let (&key, rest) = unsafe { (*old_key).split_first().unwrap_unchecked() };

							let mut map = Map::new();

							let inner = match rest.len() {
								0 => old_inner,
								_ => Inner::Node(Box::new(Node::KeyInner(rest, old_inner))),
							};

							*map.get_mut(key) = inner;

							Node::Map(map)
						});

						self.insert(key, value);
					}

					// Split self into three nodes `Node::Seq` => `Node::Map` => `Node::Seq`
					Equality::SharePrefixUpTo(idx) => {
						self.split(idx);
						self.insert(key, value);
					}

					Equality::Shorter => {
						let key = &key[current_key.len()..];

						match inner {
							Inner::Value(_) => {
								let mut node = Box::new(Node::Empty);
								node.insert(key, value);

								inner.replace_with(|old| {
									let Inner::Value(value) = old else {
										unsafe { unreachable_unchecked() };
									};

									Inner::NodeWithValue((value, node))
								});
							}

							Inner::Node(node) | Inner::NodeWithValue((_, node)) => node.insert(key, value),

							_ => unsafe { unreachable_unchecked() },
						}
					}

					Equality::Longer => self.replace_with(|old| {
						let Node::KeyInner(old_key, old_inner) = old else {
							unsafe { unreachable_unchecked() }
						};

						let (prev, rest) = unsafe { (*old_key).split_at(key.len()) };
						let inner = Inner::NodeWithValue((value, Box::new(Node::KeyInner(rest, old_inner))));

						Node::KeyInner(prev, inner)
					}),

					Equality::Full => todo!(),
				}
			}

			Node::Map(ref mut map) => {
				let (&key, rest) = unsafe { key.split_first().unwrap_unchecked() };
				let inner = map.get_mut(key);

				if rest.is_empty() {
					match inner {
						Inner::None => *inner = Inner::Value(value),

						Inner::Value(cur_value) => *cur_value = value,

						Inner::Node(_) => inner.replace_with(|old| {
							let Inner::Node(node) = old else {
								unsafe { unreachable_unchecked() }
							};

							Inner::NodeWithValue((value, node))
						}),

						Inner::NodeWithValue(_) => todo!(),
					}
				} else {
					match inner {
						Inner::None => {
							*inner = Inner::Node(Box::new(Node::KeyInner(rest, Inner::Value(value))))
						}

						Inner::Value(_) => inner.replace_with(|old| {
							let Inner::Value(v) = old else {
								unsafe { unreachable_unchecked() }
							};

							let mut new_node = Box::new(Node::Empty);
							new_node.insert(rest, value);

							Inner::NodeWithValue((v, new_node))
						}),

						Inner::Node(node) | Inner::NodeWithValue((_, node)) => node.insert(rest, value),
					}
				}
			}
		}
	}

	fn split(&mut self, idx: usize) {
		self.replace_with(|old| {
			let Node::KeyInner(old_key, old_inner) = old else {
				unsafe { unreachable_unchecked() }
			};

			let (key, rest) = unsafe { (*old_key).split_at(idx) };

			let new_inner = {
				let (&key, rest) = unsafe { rest.split_first().unwrap_unchecked() };

				let mut map = Map::new();

				match rest.len() {
					0 => {
						*map.get_mut(key) = old_inner;
					}
					_ => {
						*map.get_mut(key) = Inner::Node(Box::new(Node::KeyInner(rest, old_inner)));
					}
				};

				Inner::Node(Box::new(Node::Map(map)))
			};

			Node::KeyInner(key, new_inner)
		})
	}
}

impl<T> Radix<T> {
	#[inline]
	pub fn new() -> Self {
		Self {
			root: Node::Map(Map::new()),
		}
	}

	#[inline]
	pub fn insert<K: AsRef<[u8]>>(&mut self, key: K, value: T) {
		let key = key.as_ref();

		self.root.insert(key, value)
	}

	#[inline]
	pub fn get<K: AsRef<[u8]>>(&self, key: K) -> Option<&T> {
		let key = key.as_ref();

		self.root.get(key)
	}
}

impl Compare for *const [u8] {
	fn compare(self, other: Self) -> Equality {
		type Int = u32;
		const SHIFT: Int = (size_of::<Int>() - 1).count_ones() as Int;

		let self_len = self.len();
		let other_len = other.len();
		let min_len = self_len.min(other_len);
		let chunks = min_len >> SHIFT;

		let mut a = unsafe { (*self).as_ptr() };
		let mut b = unsafe { (*other).as_ptr() };

		for i in 0..chunks {
			let res = unsafe { a.cast::<Int>().read_unaligned() ^ b.cast::<Int>().read_unaligned() };

			if res != 0 {
				let byte_offset = {
					if res & 0x000000ff > 0 {
						0
					} else if res & 0x0000ff00 > 0 {
						1
					} else if res & 0x00ff0000 > 0 {
						2
					} else {
						3
					}
				};

				let idx = (i << SHIFT) + byte_offset;

				return if idx == 0 {
					Equality::None
				} else {
					Equality::SharePrefixUpTo(idx)
				};
			}

			a = unsafe { a.add(1 << SHIFT) };
			b = unsafe { b.add(1 << SHIFT) };
		}

		for i in chunks << SHIFT..min_len {
			if unsafe { *a != *b } {
				return if i == 0 {
					Equality::None
				} else {
					Equality::SharePrefixUpTo(i)
				};
			}

			a = unsafe { a.add(1) };
			b = unsafe { b.add(1) };
		}

		match self_len.cmp(&other_len) {
			std::cmp::Ordering::Equal => Equality::Full,
			std::cmp::Ordering::Less => Equality::Shorter,
			std::cmp::Ordering::Greater => Equality::Longer,
		}
	}
}

impl<T> std::fmt::Debug for Node<T>
where
	T: std::fmt::Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_tuple("Node")
			.field_with(|f| match self {
				Node::KeyInner(k, inner) => f
					.debug_tuple("KeyInner")
					.field(unsafe { &std::str::from_utf8_unchecked(&**k) })
					.field(&inner)
					.finish(),

				Node::Map(map) => f.debug_tuple("Map").field(map).finish(),

				Node::Empty => Ok(()),
			})
			.finish()
	}
}

trait Replacer {
	#[inline]
	fn replace_with<F>(&mut self, with: F)
	where
		F: FnOnce(Self) -> Self,
		Self: Sized,
	{
		unsafe {
			let old = std::ptr::read(self);

			let new = with(old);

			std::ptr::write(self, new);
		}
	}
}

impl<T> Replacer for Node<T> {}
impl<T> Replacer for Inner<T> {}

mod map;
