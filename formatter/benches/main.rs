#![feature(test)]

extern crate test;

use test::Bencher;
use utils::{get_test_cases, Case};

use hel_css_formatter::*;

struct Void;

#[bench]
#[ignore = "use -- --ignored --nocapture"]
fn main(b: &mut Bencher) {
	let Case {
		before,
		after,
		name,
		..
	} = get_test_cases().find(|case| case.complexity == 4).unwrap();
	let mut res = Vec::new();

	eprintln!("Running benches for - {name}");

	assert!(before.len() > 100000);

	format(&before, &mut res).expect("formatted result");

	assert_eq!(res, after.as_bytes());

	b.iter(|| format(before.as_str(), &mut Void {}))
}

impl std::io::Write for Void {
	#[inline(always)]
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
		Ok(buf.len())
	}

	#[inline(always)]
	fn flush(&mut self) -> std::io::Result<()> {
		Ok(())
	}
}

#[path = "../tests/utils.rs"]
mod utils;
