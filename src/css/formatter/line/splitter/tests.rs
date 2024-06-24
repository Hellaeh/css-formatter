use hel_colored::Colored;

use super::{Split, Splitter};

#[test]
fn small() {
	fn helper(subject: &str) {
		use std::str::from_utf8_unchecked as to_str;

		let Split { offset, bytes } = Splitter::split(subject.as_bytes())
			.next()
			.unwrap_or_else(|| panic!("{}", "empty result".red().to_string()));

		let s = unsafe { to_str(bytes) };

		assert_eq!((offset, s), (0, subject));
	}

	helper(r#"fn(")")"#);
	helper(r#"fn(a, b, c)"#);
}

#[derive(Debug)]
struct TestCase {
	// Bytes of valid and formatted CSS
	buf: &'static str,
	expected: &'static [Split<&'static str>],
}

enum Error {
	ByteMismatchAt(usize),
	BufLenDifference(isize),

	ActualShorter(usize),
	ActualLonger(usize),

	Offset,
}

impl<'a> Split<&'a [u8]> {
	const fn as_str(&self) -> &Split<&'a str> {
		unsafe { std::mem::transmute(self) }
	}
}

impl TestCase {
	const fn new(buf: &'static str, expected: &'static [Split<&'static str>]) -> Self {
		Self { buf, expected }
	}

	const fn buf(&self) -> &[u8] {
		self.buf.as_bytes()
	}

	const fn out_bytes(&self) -> &'static [Split<&'static [u8]>] {
		unsafe { std::mem::transmute(self.expected) }
	}

	fn equal_or_fail<'a, Iter>(&self, actual: Iter)
	where
		Iter: Iterator<Item = Split<&'a [u8]>>,
	{
		let actual_vec: Vec<_> = actual.collect();
		let mut actual_iter = actual_vec.iter();
		let mut expected_iter = self.out_bytes().iter();
		let mut count = 0;

		let err = 'outer: loop {
			let (actual, expected) = match (actual_iter.next(), expected_iter.next()) {
				(None, None) => return,

				(None, Some(_)) | (Some(_), None) => {
					break Error::BufLenDifference(
						(actual_iter.count() + count) as isize - self.expected.len() as isize,
					)
				}
				(Some(actual), Some(expected)) => (actual, expected),
			};

			let max_len = expected.bytes.len().max(actual.bytes.len());
			for i in 0..max_len {
				match (actual.bytes.get(i), expected.bytes.get(i)) {
					(None, Some(_)) => break 'outer Error::ActualShorter(i),
					(Some(_), None) => break 'outer Error::ActualLonger(i),
					(Some(actual), Some(expected)) => {
						if actual != expected {
							break 'outer Error::ByteMismatchAt(i);
						}
					}
					_ => {}
				};
			}

			if expected.offset != actual.offset {
				break Error::Offset;
			}

			count += 1;
		};

		eprintln!("{}", "Error!".red());
		eprintln!("\nFor '{}'", self.buf.orange());

		for i in 0..count {
			eprintln!("> {}", self.expected[i]);
		}

		eprintln!("> {}", self.expected[count].to_string().on_rgb(0, 60, 0));

		match err {
			Error::ByteMismatchAt(idx) => {
				// todo!();
				let actual = actual_vec[count].as_str().to_string().on_rgb(60, 0, 0);
				eprintln!("x {actual}",);
			}

			Error::BufLenDifference(diff) => {
				let msg = format!(
					"Actual is {} by {}",
					if diff < 0 { "shorter" } else { "longer" },
					diff.abs()
				)
				.orange();

				eprintln!("{}", msg);
			}

			Error::ActualShorter(_) => {
				let actual = actual_vec[count].as_str().to_string().on_rgb(60, 0, 0);
				eprintln!("x {actual}",);
			}

			Error::ActualLonger(_) => {
				let actual = actual_vec[count].as_str().to_string().on_rgb(60, 0, 0);
				eprintln!("x {actual}",);
			}

			Error::Offset => {
				let s = &actual_vec[count];
				todo!()
			}
		}

		panic!();
	}
}

impl<T: std::fmt::Display> std::fmt::Display for Split<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		for _ in 0..self.offset {
			f.write_str("    ")?;
		}

		write!(f, "{}", self.bytes)
	}
}

#[test]
fn string_split() {
	macro_rules! helper {
			($buf: expr, $(($offset: expr, $str: expr)$(,)?)*) => {
				TestCase::new($buf, &[
					$(Split { offset: $offset, bytes: $str }),*
				])
			};
		}

	let cases: &[TestCase] = &[
		helper!(
			".u-header--bg-transparent.u-header--bordered:not(.bg-white, .js-header-fix-moment, .u-scrolled) .u-header__section,",
			(0, ".u-header--bg-transparent.u-header--bordered:not("),
			(1, ".bg-white,")
			(1, ".js-header-fix-moment,")
			(1, ".u-scrolled")
			(0, ") .u-header__section,")
		),
		
		helper!(
			r#"grid-template-areas: "featured minor-benefit-0" "featured minor-benefit-1" "major-benefit-1 major-benefit-0";"#,
			(0, "grid-template-areas:"),
			(1, r#""featured minor-benefit-0""#),	
			(1, r#""featured minor-benefit-1""#),	
			(1, r#""major-benefit-1 major-benefit-0";"#),	
			// (1, r#""featured minor-benefit-0" "featured minor-benefit-1""#),
			// (2, r#""major-benefit-1 major-benefit-0";"#),	
		),
		
		helper!(
			"super deep and long selector that you can break into smaller parts and also exceeds any length limit possible {",
			(0, "super deep and long selector that you can break into smaller parts and also"),
			(1, "exceeds any length limit possible"),
			(0, "{")
		),

		helper!(
			r".input-group > .input-group-append:last-child > .btn:not(:last-child):not(.dropdown-toggle),",
			(0, ".input-group > .input-group-append:last-child"),
			(1, "> .btn:not(:last-child):not(.dropdown-toggle),"),
		),

		helper!(
			r".input-group > .input-group-append:last-child > .btn:not(:last-child):not(.dropdown-toggle) {",
			(0, ".input-group > .input-group-append:last-child"),
			(1, "> .btn:not(:last-child):not(.dropdown-toggle)"),
			(0, "{")
		),

		helper!(
			r"background: linear-gradient(90.33deg, rgba(32, 145, 251, 0.2), rgba(1, 153, 255, 0.09));",
			(0, "background:"),
			(1, "linear-gradient(90.33deg, rgba(32, 145, 251, 0.2), rgba(1, 153, 255, 0.09));"),
		),

		helper!(
			r#"a:is([href*="path1"]:not([href~="path3"]), [href*="path2"]:not([href~="path4"])) {"#,
			(0, "a:is("),
			(1, r#"[href*="path1"]:not([href~="path3"]),"#),
			(1, r#"[href*="path2"]:not([href~="path4"])"#),
			(0, ") {"),
		),

		helper!(
			"background: conic-gradient(from 230deg at 51.63% 52%, rgb(36, 0, 255) 0deg, rgb(0, 135, 255) 65deg, rgb(154, 25, 246) 198.75deg, rgb(15, 33, 192) 255deg, rgb(84, 135, 229) 300deg, rgb(108, 49, 226) 360deg);",
			(0, "background:"),
			(1, "conic-gradient("),
			(2, "from 230deg at 51.63% 52%,"),
			(2, "rgb(36, 0, 255) 0deg,"),
			(2, "rgb(0, 135, 255) 65deg,"),
			(2, "rgb(154, 25, 246) 198.75deg,"),
			(2, "rgb(15, 33, 192) 255deg,"),
			(2, "rgb(84, 135, 229) 300deg,"),
			(2, "rgb(108, 49, 226) 360deg"),
			(1, ");"),
		),

		helper!(
			r#"--font: Inter, -system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Ubuntu, sans-serif;"#,
			(0, "--font:"),
			(1, "Inter,"),
			(1, "-system,"),
			(1, "BlinkMacSystemFont,"),
			(1, r#""Segoe UI","#),
			(1, "Roboto,"),
			(1, r#""Helvetica Neue","#),
			(1, "Ubuntu,"),
			(1, "sans-serif;")
		),
	];

	for test in cases {
		test.equal_or_fail(Splitter::split(test.buf()))
	}
}
