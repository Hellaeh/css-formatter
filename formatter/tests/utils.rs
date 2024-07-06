/// FIXME: Move this to utils crate
#[allow(dead_code)]
pub fn format(css: &str) -> Result<(String, String), (String, String)> {
	use std::io::Write;
	use std::process::*;

	#[cfg(debug_assertions)]
	const PATH: &str = "../target/debug/hel-css-formatter.exe";
	#[cfg(not(debug_assertions))]
	const PATH: &str = "../target/release/hel-css-formatter.exe";

	let mut child = Command::new(PATH)
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

	let output = (
		String::from_utf8(res.stdout).unwrap(),
		String::from_utf8(res.stderr).unwrap(),
	);

	if !res.status.success() {
		return Err(output);
	}

	Ok(output)
}

pub struct Case {
	pub complexity: usize,
	pub order: usize,

	pub name: String,

	pub before: String,
	pub after: String,
}

impl std::fmt::Debug for Case {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"c: {}, o: {}, name: {}",
			self.complexity, self.order, self.name
		)
	}
}

pub fn get_test_cases() -> impl Iterator<Item = Case> {
	// We might store and sort it
	std::fs::read_dir("tests/css/")
		.expect("Cannot find css dir in tests")
		.flatten()
		// filter non directories
		.filter(|dir| dir.file_type().unwrap().is_dir())
		// read each directory by complexity
		.flat_map(|dir| {
			let dir_name = dir.file_name().into_string().unwrap();

			let complexity = dir_name
				.split_once('-')
				.unwrap()
				.0
				.parse::<usize>()
				.unwrap();

			std::fs::read_dir(dir.path())
				.unwrap()
				.flatten()
				.map(move |order_dir| (complexity, order_dir))
		})
		.flat_map(|(complexity, dir)| {
			let dir_name = dir.file_name().into_string().ok()?;
			let split = dir_name.split_once('-')?;

			let order = split.0.parse::<usize>().unwrap_or(0);
			let name = split.1.to_owned();

			let mut entries = std::fs::read_dir(dir.path()).unwrap().flatten();
			let (entry1, entry2) = (entries.next()?, entries.next()?);

			let read = |entry: std::fs::DirEntry| std::fs::read_to_string(entry.path()).unwrap();

			let (before, after) = if entry1.file_name().into_string().unwrap().contains("before") {
				(read(entry1), read(entry2))
			} else {
				(read(entry2), read(entry1))
			};

			Some(Case {
				complexity,
				order,

				name,

				before,
				after,
			})
		})
}

#[derive(Debug)]
pub struct Difference<'a> {
	pub row: usize,
	pub col: usize,
	pub actual: &'a str,
	pub expected: &'a str,
}

pub fn differentiate<'a>(result: &'a str, should_be: &'a str) -> Result<(), Difference<'a>> {
	if result == should_be {
		return Ok(());
	}

	for (row, line) in result.lines().zip(should_be.lines()).enumerate() {
		let (long, short) = if line.0.len() >= line.1.len() {
			(line.0, line.1)
		} else {
			(line.1, line.0)
		};

		for i in 0..long.len() {
			let long_byte = unsafe { long.as_bytes().get_unchecked(i) };

			if let Some(short_byte) = short.as_bytes().get(i) {
				if long_byte == short_byte {
					continue;
				}
			}

			return Err(Difference {
				row,
				col: i,
				actual: line.0,
				expected: line.1,
			});
		}

		// if total != count {
		// 	return
		// }
	}

	// CRLF vs LF difference ignored
	Ok(())
}

pub fn format_error_message(should_be: &str, diff: Difference<'_>) -> String {
	use hel_colored::Colored;
	use std::fmt::Write;

	const LINES: usize = 3;

	let mut res = String::new();

	let skip = (diff.row + 1).saturating_sub(LINES);
	let mut current_line = skip + 1;

	let mut lines = should_be.lines().skip(skip);

	let num_width = skip.checked_ilog10().unwrap_or(0) as usize;

	for line in lines.by_ref().take(diff.row - skip) {
		writeln!(
			&mut res,
			"{}",
			&format!("{current_line:0num_width$} > {}", line)
				.on_rgb(0, 60, 0)
				.to_string()
		)
		.unwrap();

		current_line += 1;
	}

	writeln!(
		&mut res,
		"{}",
		&format!("{current_line:0num_width$} > {}", diff.expected)
			.on_rgb(0, 60, 0)
			.to_string()
	)
	.unwrap();

	writeln!(
		&mut res,
		"{}",
		&format!("{current_line:0num_width$} x {}", diff.actual)
			.on_rgb(60, 0, 0)
			.to_string()
	)
	.unwrap();

	current_line += 1;

	for line in lines.by_ref().skip(1).take(LINES) {
		writeln!(
			&mut res,
			"{}",
			&format!("{current_line:0num_width$} > {}", line)
				.on_rgb(0, 60, 0)
				.to_string()
		)
		.unwrap();

		current_line += 1;
	}

	res
}
