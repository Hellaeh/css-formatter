use utils::*;

#[test]
fn test_all_except_big() {
	use hel_colored::Colored;

	let mut failed = 0;

	for (num, (name, before, after)) in get_test_cases().enumerate() {
		// if name.contains("big") {
		// 	continue;
		// }

		print!(
			"{}",
			format!("Test {}: {} - ", num + 1, name.yellow()).green()
		);

		match format(&before) {
			Ok((stdout, _)) => match differentiate(&stdout, &after) {
				Ok(_) => println!("{}", "Passed!".green()),

				Err(diff) => {
					println!("{}", "Failed!".red());

					println!("{}", format_error_message(&after, diff));

					failed += 1;
				}
			},
			Err((_, stderr)) => {
				println!("{}", "Failed!".red());
				println!("stderr:\n{}\n", stderr.orange());

				failed += 1;
			}
		}
	}

	if failed > 0 {
		panic!("{}", format!("Failed {} tests!", failed).red());
	}
}

mod utils;
