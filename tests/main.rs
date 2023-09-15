fn format(css: &str) -> (String, String) {
	use std::io::Write;
	use std::process::*;

	let mut child = Command::new("cargo")
		.args(["run"])
		.stdout(Stdio::piped())
		.stderr(Stdio::piped())
		.stdin(Stdio::piped())
		.spawn()
		.expect("Could not spawn a child process");

	child
		.stdin
		.as_mut()
		.expect("Could not take child's stdin")
		.write_all(css.as_bytes())
		.expect("Failed to write to child's stdin");

	let res = child.wait_with_output().expect("Failed to wait on child");

	(
		String::from_utf8(res.stdout).unwrap(),
		String::from_utf8(res.stderr).unwrap(),
	)
}

fn get_test_cases_except_big() -> impl Iterator<Item = (String, String)> {
	std::fs::read_dir("tests/css/")
		.expect("Cannot find css dir in tests")
		.flatten()
		// filter non directories
		.filter(|dir| dir.file_type().unwrap().is_dir())
		// filter "big"
		.filter(|dir| dir.file_name() != "big")
		// read each directory
		.flat_map(|dir| std::fs::read_dir(dir.path()))
		// read files
		.flat_map(|mut entries| {
			let (Some(Ok(entry1)), Some(Ok(entry2))) = (entries.next(), entries.next()) else {
				return None;
			};

			let read = |entry: std::fs::DirEntry| std::fs::read_to_string(entry.path()).unwrap();

			if entry1.file_name().into_string().unwrap().contains("before") {
				Some((read(entry1), read(entry2)))
			} else {
				Some((read(entry2), read(entry1)))
			}
		})
}

#[test]
fn test_all_except_big() {
	use hel_colored::Colored;

	for (num, (before, after)) in get_test_cases_except_big().enumerate() {
		println!("\n{}", format!("Test {}", num + 1).green());

		let (before, stderr) = format(&before);

		assert_eq!(before, after, "\n{}:\n{}", "Failed!".red(), stderr.orange());
	}
}
