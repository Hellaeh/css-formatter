pub fn format(css: &str) -> Result<(String, String), (String, String)> {
	use std::io::Write;
	use std::process::*;

	#[cfg(debug_assertions)]
	const PATH: &str = "./target/debug/hel-css-formatter.exe";
	#[cfg(not(debug_assertions))]
	const PATH: &str = "./target/release/hel-css-formatter.exe";

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

pub fn get_test_cases() -> impl Iterator<Item = (String, String, String)> {
	std::fs::read_dir("tests/css/")
		.expect("Cannot find css dir in tests")
		.flatten()
		// filter non directories
		.filter(|dir| dir.file_type().unwrap().is_dir())
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

#[derive(Debug)]
pub struct Difference<'a> {
	pub row: usize,
	pub col: usize,
	pub line_wrong: &'a str,
	pub line_correct: &'a str,
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
				line_wrong: line.0,
				line_correct: line.1,
			});
		}

		// if total != count {
		// 	return
		// }
	}

	// CRLF vs LF difference ignored
	Ok(())
}

pub fn format_error_message<'a>(should_be: &'a str, diff: Difference<'a>) -> String {
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
				.on_rgb((0, 60, 0))
				.to_string()
		)
		.unwrap();

		current_line += 1;
	}

	writeln!(
		&mut res,
		"{}",
		&format!("{current_line:0num_width$} > {}", diff.line_correct)
			.on_rgb((0, 60, 0))
			.to_string()
	)
	.unwrap();

	writeln!(
		&mut res,
		"{}",
		&format!("{current_line:0num_width$} x {}", diff.line_wrong)
			.on_rgb((60, 0, 0))
			.to_string()
	)
	.unwrap();

	current_line += 1;

	for line in lines.by_ref().skip(1).take(LINES) {
		writeln!(
			&mut res,
			"{}",
			&format!("{current_line:0num_width$} > {}", line)
				.on_rgb((0, 60, 0))
				.to_string()
		)
		.unwrap();

		current_line += 1;
	}

	res
}
