use split_tree::{Kind, SplitNode, SplitTree};

use crate::css::formatter::line::MAX_LENGTH;

use consts::ASCII;

#[derive(Debug, Clone)]
pub(crate) struct Split<T> {
	pub offset: u8,
	pub bytes: T,
}

pub struct Splitter;

impl Splitter {
	/// Split any ASCII byte array into smaller chunks.
	///
	/// Kind of expensive
	#[inline]
	pub fn split(buf: &[u8]) -> impl Iterator<Item = Split<&[u8]>> {
		if buf.starts_with(&[ASCII::SLASH, ASCII::ASTERISK]) {
			panic!("Comments are not allowed here");
		}

		#[cfg(debug_assertions)]
		{
			eprintln!("Split on:\n`{}`", unsafe {
				std::str::from_utf8_unchecked(buf)
			});
		}

		let splits = SplitTree::from_byte_array(buf);

		if splits.is_empty() {
			// TODO: Implement early return type ... enum?
			panic!("No splits found");
		}

		Self::flatten(buf, splits);

		splits.into_iter().map_windows(move |[a, b]| {
			let from = (a.at as isize + a.offset_from as isize) as usize;
			let to = (b.at as isize + b.offset_to as isize) as usize;

			let bytes = &buf[from..to];

			if bytes.len() > MAX_LENGTH {
				eprintln!("Could not break {} len string", bytes.len())
			}

			Split {
				offset: a.offset_indent,
				bytes: &buf[from..to],
			}
		})
	}

	#[inline]
	fn flatten(buf: &[u8], splits: SplitTree) {
		Self::flatten_groups(buf, splits);
		Self::remove_soft_splits(buf, splits);
		Self::remove_conditional(buf, splits);
	}

	#[inline]
	fn flatten_groups(buf: &[u8], splits: SplitTree) {
		for node in splits
			.iter_mut()
			.filter(|s| matches!(s.kind, Kind::ClosedGroupStart))
		{
			let group_len = unsafe { node.calc_group_len() };

			if group_len > MAX_LENGTH {
				continue;
			}

			let next = splits[node.metadata as usize + 1];
			let prev_at = node.prev().map(|s| s.at as usize).unwrap_or(0);
			let mut next_at = next.at as usize;

			if next.is_plug() || matches!(next.kind, Kind::Conditional) {
				next_at = buf.len();
			}

			if next_at - prev_at > MAX_LENGTH {
				continue;
			}

			unsafe { node.collapse_closed_group() };
		}
	}

	#[inline]
	fn remove_soft_splits(buf: &[u8], splits: SplitTree) {
		macro_rules! iter_filter {
			($kind: pat) => {
				splits
					.iter_mut()
					.filter(|s| !s.is_plug() && matches!(s.kind, $kind))
			};
		}

		#[inline]
		fn inner<'a>(buf: &[u8], soft_splits: impl Iterator<Item = &'a mut SplitNode>) {
			for split in soft_splits {
				let prev = split.prev().map(|s| s.at as usize).unwrap_or(0);
				let next = split.next().map(|s| s.at as usize).unwrap_or(buf.len());

				if next - prev < MAX_LENGTH {
					split.remove();
				}
			}
		}

		// FIXME: Do a proper algorithm
		if !buf.starts_with(b"grid-template-areas") {
			inner(buf, iter_filter!(Kind::Whitespace));
		}
		inner(buf, iter_filter!(Kind::Operator));
	}

	#[inline]
	fn remove_conditional(_: &[u8], splits: SplitTree) {
		let Some(penultimate) = splits.last().and_then(|s| s.prev()) else {
			return;
		};

		if !matches!(penultimate.kind, Kind::Conditional) {
			return;
		}

		let Some(prev) = penultimate.prev() else {
			penultimate.remove();
			return;
		};

		if prev.offset_indent != penultimate.offset_indent {
			return;
		}

		penultimate.remove();
	}
}

mod split_tree;

#[cfg(test)]
mod tests;
