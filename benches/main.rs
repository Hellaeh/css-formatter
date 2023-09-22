#![feature(test)]

extern crate test;

use test::Bencher;
use utils::*;

#[bench]
fn sorting(b: &mut Bencher) {
	let (_, before, after) = get_test_cases()
		.find(|(name, _, _)| name.contains("sort"))
		.unwrap();

	differentiate(&format(&before).expect("Failed!").0, &after).expect("Failed!");

	// We're practically benching how fast OS can pipe.
	// I suppose we'll use it as baseline in future
	b.iter(|| format(&before));
}

#[path = "../tests/utils.rs"]
mod utils;
