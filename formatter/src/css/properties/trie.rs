use radix::Radix;

use super::{Descriptor, Property};

type Local<T> = Radix<T>;

#[derive(Debug)]
pub struct Trie {
	inner: Local<Descriptor>,
}

impl Trie {
	#[inline]
	pub fn new() -> Self {
		let mut inner = Local::new();

		for i in 0..std::mem::variant_count::<Property>() {
			let prop = unsafe { std::mem::transmute::<u16, Property>(i as u16) };
			let desc = prop.descriptor();
			inner.insert(desc.name(), desc);
		}

		Self { inner }
	}
}

impl std::ops::Deref for Trie {
	type Target = Local<Descriptor>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
