use super::{Descriptor, Property};

#[derive(Debug)]
pub struct Trie<'a> {
	inner: hel_trie::Trie<Descriptor<'a>>,
}

impl<'a> Trie<'a> {
	/// Constructs a new empty trie
	#[inline]
	pub fn new() -> Self {
		let mut inner = hel_trie::Trie::new();

		for i in 0..std::mem::variant_count::<Property>() {
			let prop = unsafe { std::mem::transmute::<u16, Property>(i as u16) };
			let desc = prop.descriptor();
			inner.insert(desc.name(), desc);
		}

		Self { inner }
	}
}

impl<'a> std::ops::Deref for Trie<'a> {
	type Target = hel_trie::Trie<Descriptor<'a>>;

	#[inline]
	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
