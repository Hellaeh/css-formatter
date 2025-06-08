use std::simd::{cmp::SimdPartialEq, Simd};

use consts::ASCII;

use crate::{Error, Result, Token, Tokenizer, LANE_WIDTH, LANE_WIDTH_MASK};

#[inline]
pub fn tokenize_comment_simd(tokenizer: &mut Tokenizer) -> Result {
	macro_rules! token {
		($ptr: expr) => {
			Ok(Token::from(Token::Comment, $ptr, tokenizer.cursor))
		};
	}

	let at_ptr = tokenizer.cursor;
	// go back to alignment bounds, if not aligned
	let misalignment = at_ptr.addr() & LANE_WIDTH_MASK;
	// skip every match before cursor
	let mut skip_mask = u64::MAX << misalignment;
	let mut aligned_ptr = unsafe { at_ptr.sub(misalignment) };

	// vector
	while aligned_ptr.addr() + LANE_WIDTH <= tokenizer.eof.addr() {
		// this probably won't work on big endian
		const ASTERISKS_MASK: Simd<u8, LANE_WIDTH> = Simd::splat(b'*');
		const SLASHES_MASK: Simd<u8, LANE_WIDTH> = Simd::splat(b'/');
		const BORDER_STAR: u64 = 1 << LANE_WIDTH_MASK;

		// noop
		let chunk = aligned_ptr as *const [u8; LANE_WIDTH];

		let op = unsafe { Simd::from_array(*chunk) };

		// should be good for OoOE
		let asterisks = op.simd_eq(ASTERISKS_MASK).to_bitmask();
		let slashes = op.simd_eq(SLASHES_MASK).to_bitmask();
		let shifted_slashed = slashes >> 1;

		let result = asterisks & shifted_slashed & skip_mask;

		if result > 0 {
			tokenizer.cursor = unsafe { aligned_ptr.add(result.trailing_zeros() as usize) };
			return token!(at_ptr);
		}

		// aligned_ptr = unsafe { aligned_ptr.add(LANE_WIDTH) };
		// REASON: i want this to be fast even with `#[cfg(debug_assertions)]` or non-release build
		// could technically overflow tho, but highly unlikely
		aligned_ptr = (aligned_ptr.addr() + LANE_WIDTH) as *const u8;

		// check comment span over chunk border
		if asterisks & BORDER_STAR > 0 && aligned_ptr < tokenizer.eof && unsafe { *aligned_ptr } == b'/'
		{
			tokenizer.cursor = unsafe { aligned_ptr.sub(1) };
			return token!(at_ptr);
		}

		skip_mask = u64::MAX;
	}

	tokenizer.cursor = aligned_ptr;

	// scalar
	while !tokenizer.is_eof() {
		let ch = tokenizer.current_byte();

		if ch == ASCII::ASTERISK {
			let Some(next) = tokenizer.try_peek_next_byte() else {
				// EOF
				break;
			};

			if next == ASCII::SLASH {
				return token!(at_ptr);
			}
		}

		tokenizer.advance(1);
	}

	Err(Error::BadComment)
}

#[cfg(test)]
mod test {
	use consts::ASCII;

	use super::tokenize_comment_simd;
	use crate::{simd::utils::copy_to_aligned, Error, Result, Token, Tokenizer, LANE_WIDTH_MASK};

	fn scalar<S: AsRef<[u8]>>(bytes: S) -> Result {
		let bytes = bytes.as_ref();

		for (i, seq) in bytes.windows(2).enumerate() {
			if !matches!(seq, &[ASCII::ASTERISK, ASCII::SLASH]) {
				continue;
			}

			let ptr = bytes.as_ptr();

			return Ok(Token::from(Token::Comment, ptr, unsafe { ptr.add(i) }));
		}

		Err(Error::BadComment)
	}

	fn helper<S: AsRef<[u8]>>(input: S) {
		let input = copy_to_aligned(input);

		let mut tokenizer = Tokenizer::new(input);
		let scalar_res = scalar(input);
		let simd_res = tokenize_comment_simd(&mut tokenizer);

		assert_eq!(scalar_res, simd_res);
	}

	#[test]
	fn main() {
		helper(" comment */");
		helper(" invalid comment no closing * /");
		helper("***this is inside a comment/*** div {} /*/ ")
	}

	#[test]
	fn span_over_chunk_border() {
		let len = LANE_WIDTH_MASK;
		let lane_fill = "*".repeat(len);
		let subject = format!("{lane_fill}*/ div {{}}");

		helper(subject);
	}
}
