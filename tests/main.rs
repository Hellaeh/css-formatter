use utils::*;

#[test]
fn test_all() {
	use hel_colored::Colored;
	use hel_thread_pool::ThreadPool;

	let pool = ThreadPool::with_capacity(4);
	let (tx, rx) = std::sync::mpsc::channel();

	let mut total = 0;

	for (num, case) in get_test_cases().enumerate() {
		let tx = tx.clone();

		#[allow(unused_must_use)]
		pool.execute(move || {
			let first_pass = format(&case.before);

			let second_pass = if let Ok((out, _)) = &first_pass {
				let second_pass = format(out);
				Some(second_pass)
			} else {
				None
			};

			tx.send((num, case, first_pass, second_pass));
		});

		total += 1;
	}

	drop(tx);

	let mut failed = Vec::new();

	while let Ok((num, case, first, second)) = rx.recv_timeout(std::time::Duration::from_secs(10)) {
		let Case {
			complexity,
			order,
			name,
			after,
			..
		} = &case;

		// Ignore bigger tests output if any small test fails
		if *complexity >= 4 && failed.len() > 0 {
			total -= 1;
			continue;
		}

		print!(
			"{}",
			format!(
				"Test {}: c{complexity} o{order} {} - ",
				num + 1,
				name.yellow()
			)
			.green()
		);

		match first {
			Ok((stdout, stderr)) => match differentiate(&stdout, &after) {
				Ok(_) => {
					match second.unwrap() {
						Ok((stdout, stderr)) => match differentiate(&stdout, &after) {
							Ok(_) => println!("{}", "Passed!".green()),
							Err(diff) => {
								let msg = format_error_message(&after, diff);

								println!("{}", "Failed second pass!".red());
								println!("stderr:\n{}", stderr.orange());

								println!("{}", msg);

								failed.push(case);
							}
						},

						Err((_, stderr)) => {
							println!("{}", "Failed second pass!".red());
							println!("stderr:\n{}", stderr.orange());

							failed.push(case);
						}
					};
				}

				Err(diff) => {
					let msg = format_error_message(&after, diff);

					println!("{}", "Failed!".red());
					println!("stderr:\n{}", stderr.orange());

					println!("{}", msg);

					failed.push(case);
				}
			},
			Err((_, stderr)) => {
				println!("{}", "Failed!".red());
				println!("stderr:\n{}", stderr.orange());

				failed.push(case);
			}
		}

		total -= 1;
	}

	if total != 0 {
		panic!("{}", "Some of the tests timed out".red());
	}

	if !failed.is_empty() {
		panic!("{}", format!("Failed:\n{:#?}", failed).red());
	}
}

mod utils;
