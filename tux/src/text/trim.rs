/// Removes extra whitespace, excess leading indentation, and normalizes line
/// breaks from the input string.
///
/// This will pass the input through [`lines`](super::lines::lines) and then
/// strip excess leading indentation from the lines before joining then again
/// using `"\n"` as a separator.
///
/// The _excess leading indentation_ is defined by the first non-blank line
/// in the input. Any indentation on that line will also be removed from the
/// prefix of all other lines.
///
/// # Example
///
/// The main purpose of this method is to generate clean strings from literal
/// strings defined in the source code:
///
/// ```
/// use tux::text::trim;
///
/// let input = r#"
///
/// 	Line 1
/// 		Line 2
/// 	Line 3
///
/// 	Line 4
///
/// "#;
///
/// let output = trim(input);
/// let expected = vec![
/// 	"Line 1",
/// 	"\tLine 2",
/// 	"Line 3",
/// 	"",
/// 	"Line 4",
/// ].join("\n");
/// assert_eq!(output, expected);
/// ```
///
pub fn trim<S: AsRef<str>>(input: S) -> String {
	let lines = super::lines(input);
	if lines.len() == 0 {
		return String::new();
	}
	let first = lines[0].as_str();
	let trimmed = first.trim_start();
	let indent_length = first.len() - trimmed.len();
	let indent = &first[..indent_length];
	let lines = lines
		.iter()
		.map(|x| x.strip_prefix(indent).unwrap_or(x.as_str()))
		.collect::<Vec<_>>();
	lines.join("\n")
}

#[cfg(test)]
mod test_trim {
	use super::trim;

	#[test]
	fn of_empty_string() {
		let out = trim("");
		assert_eq!(out, "");
	}

	#[test]
	fn removes_extra_blank_lines() {
		let out = trim("\n \n  \ntext\n\n \n  \n");
		assert_eq!(out, "text");
	}

	#[test]
	fn removes_first_line_indentation_from_all_lines() {
		let out = trim("\n  l1\n\n    l2\n  l3\n");
		assert_eq!(out, "l1\n\n  l2\nl3");
	}
}
