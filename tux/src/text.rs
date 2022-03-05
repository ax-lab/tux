//! Text utilities for tests.

/// Splits the input into lines while trimming extraneous white-space.
///
/// This will trim spaces from the end of lines, and remove leading and
/// trailing empty lines.
pub fn lines<S: AsRef<str>>(input: S) -> Vec<String> {
	let input = input.as_ref().trim_end();
	let iter = LinesIterator { input, index: 0 };
	iter.map(|x| x.trim_end())
		.skip_while(|x| x.len() == 0)
		.map(|x| x.into())
		.collect()
}

struct LinesIterator<'a> {
	input: &'a str,
	index: usize,
}

impl<'a> Iterator for LinesIterator<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		let input = &self.input[self.index..];
		let bytes = input.as_bytes();
		if bytes.len() == 0 {
			None
		} else if let Some(index) = input.find(&['\n', '\r']) {
			let output = &input[..index];
			let next_index = if bytes[index] == '\r' as u8
				&& index < bytes.len() - 1
				&& bytes[index + 1] == '\n' as u8
			{
				index + 2
			} else {
				index + 1
			};
			self.index += next_index;
			Some(output)
		} else {
			self.index += input.len();
			Some(input)
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lines_returns_single_line_for_no_line_break() {
		let out = lines("single line");
		assert_eq!(out, vec!["single line"]);
	}

	#[test]
	fn lines_supports_multiple_line_break_sequences() {
		let out = lines("line 1\nline 2\r\nline 3\rline 4");
		assert_eq!(out, vec!["line 1", "line 2", "line 3", "line 4"]);
	}

	#[test]
	fn lines_returns_empty_for_empty_string() {
		let out = lines("");
		assert!(out.len() == 0);
	}

	#[test]
	fn lines_skips_leading_empty_lines() {
		let out = lines("\n\n  \r\n\t\rfirst non-empty\nsecond");
		assert_eq!(out, vec!["first non-empty", "second"]);
	}

	#[test]
	fn lines_removes_trailing_empty_lines() {
		let out = lines("first\nlast non-empty\n\n  \r\n\t\r\n");
		assert_eq!(out, vec!["first", "last non-empty"]);
	}

	#[test]
	fn lines_trim_line_ends() {
		let out = lines("l1  \nl2\t\r\nl3\t \rl4  ");
		assert_eq!(out, vec!["l1", "l2", "l3", "l4"]);
	}
}
