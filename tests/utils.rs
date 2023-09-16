pub fn format(css: &str) -> (String, String) {
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

pub fn get_test_cases_except_big() -> impl Iterator<Item = (String, String, String)> {
	std::fs::read_dir("tests/css/")
		.expect("Cannot find css dir in tests")
		.flatten()
		// filter non directories
		.filter(|dir| dir.file_type().unwrap().is_dir())
		// filter "big"
		.filter(|dir| dir.file_name() != "big")
		// read each directory
		.map(|dir| (dir.file_name().into_string(), std::fs::read_dir(dir.path())))
		// read files
		.flat_map(|args| {
			let (Ok(name), Ok(mut entries)) = args else {
				return None;
			};

			let (Some(Ok(entry1)), Some(Ok(entry2))) = (entries.next(), entries.next()) else {
				return None;
			};

			let read = |entry: std::fs::DirEntry| std::fs::read_to_string(entry.path()).unwrap();

			if entry1.file_name().into_string().unwrap().contains("before") {
				Some((name, read(entry1), read(entry2)))
			} else {
				Some((name, read(entry2), read(entry1)))
			}
		})
}

pub struct Difference<'a> {
	pub row: usize,
	pub col: usize,
	pub left: &'a str,
	pub right: &'a str,
}

pub fn differentiate<'a>(left: &'a str, right: &'a str) -> Result<(), Difference<'a>> {
	if left == right {
		return Ok(());
	}

	for (row, line) in left.lines().zip(right.lines()).enumerate() {
		for (col, byte) in line.0.bytes().zip(line.1.bytes()).enumerate() {
			if byte.0 != byte.1 {
				return Err(Difference {
					row: row + 1,
					col: col + 1,
					left: line.0,
					right: line.1,
				});
			}
		}
	}

	unreachable!("Check line ending '\\r\\n' vs '\\n'")
}
