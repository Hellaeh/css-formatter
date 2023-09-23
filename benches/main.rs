#![feature(test)]

extern crate test;

use hel_colored::Colored;
use test::Bencher;
use utils::*;

#[bench]
#[ignore = "use -- --ignored --nocapture"]
fn main(b: &mut Bencher) {
	let (_, before, after) = get_test_cases()
		.find(|(name, _, _)| name.contains("big"))
		.unwrap();

	let baseline_case = "p{color:red;}";

	differentiate(
		&format(baseline_case).expect("Failed!").0,
		"p {\n\tcolor: red;\n}",
	)
	.expect("Failed!");

	differentiate(&format(&before).expect("Failed!").0, &after).expect("Failed!");

	let baseline = b
		.bench(|b| {
			b.iter(|| format(baseline_case));
			Ok(())
		})
		.unwrap()
		.unwrap();

	let main = b
		.bench(|b| {
			b.iter(|| format(&before));
			Ok(())
		})
		.unwrap()
		.unwrap();

	println!("{}", "Results:".green());
	println!("Min: {}", main.min - baseline.min);
	println!("Max: {}", main.max - baseline.max);
	println!("Mean: {}", main.mean - baseline.mean);
	println!("Median: {}", main.median - baseline.median);
	println!();
}

#[path = "../tests/utils.rs"]
mod utils;
