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
			let res = format(&case.before);
			tx.send((num, case, res));
		});

		total += 1;
	}

	drop(tx);

	let mut failed = 0;

	while let Ok((num, case, res)) = rx.recv_timeout(std::time::Duration::from_secs(30)) {
		let Case {
			complexity,
			order,
			name,
			after,
			..
		} = case;

		if complexity == 4 && failed > 0 {
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

		match res {
			Ok((stdout, _)) => match differentiate(&stdout, &after) {
				Ok(_) => println!("{}", "Passed!".green()),

				Err(diff) => {
					let w = format_error_message(&after, diff);
					println!("{}", "Failed!".red());
					println!("{}", w);

					failed += 1;
				}
			},
			Err((_, stderr)) => {
				println!("{}", "Failed!".red());
				println!("stderr:\n{}\n", stderr.orange());

				failed += 1;
			}
		}

		total -= 1;
	}

	if total != 0 {
		panic!("{}", "Some of the tests timed out".red());
	}

	if failed > 0 {
		panic!("{}", format!("Failed {} tests!", failed).red());
	}
}

mod utils;
