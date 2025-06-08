#[cfg(test)]
pub fn copy_to_aligned<S: AsRef<[u8]>>(bytes: S) -> &'static [u8] {
	use crate::{LANE_WIDTH, LANE_WIDTH_MASK};

	const FILL: [u8; LANE_WIDTH] = [b'-'; LANE_WIDTH];
	let bytes = bytes.as_ref();

	let old_len = bytes.len();
	let old_buf = bytes.as_ptr();

	let rem_width = LANE_WIDTH - (old_len & LANE_WIDTH_MASK);
	let new_len = old_len + rem_width;
	let layout = std::alloc::Layout::from_size_align(new_len, LANE_WIDTH).expect("layout");
	let new_buf = unsafe { std::alloc::alloc(layout) };

	if new_buf.is_null() {
		std::alloc::handle_alloc_error(layout);
	}

	unsafe {
		std::ptr::copy_nonoverlapping(old_buf, new_buf, old_len);
		std::ptr::copy_nonoverlapping(FILL.as_ptr(), new_buf.add(old_len), rem_width);

		std::slice::from_raw_parts(new_buf, new_len)
	}
}
