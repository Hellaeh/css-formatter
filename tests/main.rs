use utils::*;

#[test]
fn test_all_except_big() {
	use hel_colored::Colored;

	let mut failed = 0;

	for (num, (name, before, after)) in get_test_cases_except_big().enumerate() {
		print!(
			"\n{}",
			format!("Test {}: {}", num + 1, name.yellow()).green()
		);

		let (before, stderr) = format(&before);

		match differentiate(&before, &after) {
			Ok(_) => println!(" - {}", "Passed!".green()),

			Err(diff) => {
				println!(" - {}\n", "Failed!".red());

				println!("At line {}", diff.row);

				println!("{} {:?}", "Got:".red(), diff.left);
				println!("{} {:?}", "Should be:".green(), diff.right);

				println!("{}\n", stderr.orange());

				failed += 1;
			}
		}
	}

	if failed > 0 {
		panic!("{}", format!("Failed {} tests!", failed).red());
	}
}

mod utils;
