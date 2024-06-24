#![feature(test)]

extern crate test;

use test::Bencher;
use utils::*;

#[bench]
#[ignore = "use -- --ignored --nocapture"]
fn main(b: &mut Bencher) {
	let Case { before, .. } = get_test_cases().find(|case| case.complexity == 4).unwrap();

	b.iter(|| format(before.as_str()))
}

#[path = "../tests/utils.rs"]
mod utils;
