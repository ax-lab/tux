//! Text utilities for tests.

/// Splits the input into lines while trimming extraneous white-space.
///
/// This will trim spaces from the end of lines, and remove leading and
/// trailing empty lines.
pub fn lines<S: AsRef<str>>(input: S) -> Vec<String> {
	let input = input.as_ref().trim_end();
	let output = LinesIterator {
		input_text: input,
		cursor_pos: 0,
	};
	trim_lines(output).collect()
}

/// Removes extraneous white-space from a sequence of lines.
///
/// This will trim spaces from the end of lines, and remove leading and
/// trailing empty lines.
pub fn trim_lines<'a, I>(input: I) -> impl Iterator<Item = String>
where
	I: IntoIterator,
	I::Item: AsRef<str>,
{
	let output = input
		.into_iter()
		.map(|x| x.as_ref().trim_end().to_string())
		.skip_while(|x| x.len() == 0);
	TrimEndIterator::wrap(output)
}

struct LinesIterator<'a> {
	input_text: &'a str,
	cursor_pos: usize,
}

impl<'a> Iterator for LinesIterator<'a> {
	type Item = &'a str;

	fn next(&mut self) -> Option<Self::Item> {
		let remaining_input = &self.input_text[self.cursor_pos..];
		if remaining_input.len() == 0 {
			None
		} else if let Some(next_line_break_offset) = remaining_input.find(&['\n', '\r']) {
			let bytes = remaining_input.as_bytes();
			let output = &remaining_input[..next_line_break_offset];
			let is_carriage_return = bytes[next_line_break_offset] == '\r' as u8;
			let is_crlf_sequence = is_carriage_return
				&& next_line_break_offset < bytes.len() - 1
				&& bytes[next_line_break_offset + 1] == '\n' as u8;
			let next_cursor_offset = if is_crlf_sequence {
				next_line_break_offset + 2
			} else {
				next_line_break_offset + 1
			};
			self.cursor_pos += next_cursor_offset;
			Some(output)
		} else {
			self.cursor_pos += remaining_input.len();
			Some(remaining_input)
		}
	}
}

struct TrimEndIterator<T: Iterator<Item = String>> {
	inner: T,
	current_run_of_empty_strings: usize,
	non_empty_string_after_run: Option<String>,
}

impl<T: Iterator<Item = String>> TrimEndIterator<T> {
	pub fn wrap(inner: T) -> Self {
		TrimEndIterator {
			inner,
			current_run_of_empty_strings: 0,
			non_empty_string_after_run: None,
		}
	}
}

impl<T: Iterator<Item = String>> Iterator for TrimEndIterator<T> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.non_empty_string_after_run.is_some() {
				if self.current_run_of_empty_strings > 0 {
					self.current_run_of_empty_strings -= 1;
					return Some(String::new());
				}
				return std::mem::take(&mut self.non_empty_string_after_run);
			}

			if let Some(next) = self.inner.next() {
				if next.len() > 0 {
					self.non_empty_string_after_run = Some(next);
				} else {
					self.current_run_of_empty_strings += 1;
				}
			} else {
				return None;
			}
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

	#[test]
	fn lines_does_not_strip_lead_indentation() {
		let out = lines("\n 1\n  2\n   3");
		assert_eq!(out, vec![" 1", "  2", "   3"]);
	}

	#[test]
	fn trim_lines_returns_non_empty_lines() {
		let input = vec!["1", "2", "3"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["1", "2", "3"]);
	}

	#[test]
	fn trim_lines_skips_leading_empty_lines() {
		let input = vec!["", "  ", "\t", "first non-empty", "second"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["first non-empty", "second"]);
	}

	#[test]
	fn trim_lines_trim_line_ends() {
		let input = vec!["l1  ", "l2\t", "l3\t ", "l4  "];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["l1", "l2", "l3", "l4"]);
	}

	#[test]
	fn trim_lines_removes_trailing_empty_lines() {
		let input = vec!["first", "last non-empty", "", "  ", "\t"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["first", "last non-empty"]);
	}

	#[test]
	fn trim_lines_does_not_remove_empty_lines_from_the_middle() {
		let input = vec!["1x", "", "2x", "", "", "3x", "", "", "", "!"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["1x", "", "2x", "", "", "3x", "", "", "", "!"]);
	}

	#[test]
	fn trim_lines_does_not_trim_lead_indentation() {
		let input = vec![" 1", "  2", "   3"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec![" 1", "  2", "   3"]);
	}
}
