/// Splits the input string into lines and cleans up excess whitespace
/// using [`trim_lines`].
///
/// The supported line separators are `"\n"`, `"\r"`, and `"\r\n"`.
pub fn lines<S: AsRef<str>>(input: S) -> Vec<String> {
	let input = input.as_ref();

	// we can't simply use the default `lines()` here because of the `\r`
	let output = LinesIterator {
		input_text: input,
		cursor_pos: 0,
	};

	trim_lines(output).collect()
}

/// Removes extra space from an input string sequence received as an iterator.
///
/// This return an iterator wrapping the original input that will:
///
/// - Remove leading and trailing blank lines. A blank line is either an
///   empty string or a string containing only spaces.
/// - Trim the end of all lines.
pub fn trim_lines<'a, I>(input: I) -> impl Iterator<Item = String>
where
	I: IntoIterator,
	I::Item: AsRef<str>,
{
	let output = input
		.into_iter()
		.map(|x| x.as_ref().trim_end().to_string())
		.skip_while(|x| x.len() == 0);

	// note that this only handles empty strings, so we need to make sure
	// lines are already trimmed before they go through this iterator
	TrimEndIterator::wrap(output)
}

/// Iterate over each line in the input text. This differs from Rust library
/// implementation in which it considers a lone `\r` as a line break.
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
			let remaining_as_bytes = remaining_input.as_bytes();
			let current_line = &remaining_input[..next_line_break_offset];

			let last_character_index = remaining_as_bytes.len() - 1;
			let is_carriage_return = remaining_as_bytes[next_line_break_offset] == '\r' as u8;
			let is_crlf_sequence = is_carriage_return
				&& next_line_break_offset < last_character_index
				&& remaining_as_bytes[next_line_break_offset + 1] == '\n' as u8;

			let next_cursor_offset = if is_crlf_sequence {
				next_line_break_offset + 2
			} else {
				next_line_break_offset + 1
			};
			self.cursor_pos += next_cursor_offset;
			Some(current_line)
		} else {
			self.cursor_pos += remaining_input.len();
			Some(remaining_input)
		}
	}
}

/// An iterator over a sequence of strings that skips empty strings from the
/// end of the iteration.
///
/// # Implementation note
///
/// This is meant to be used by [`lines`] and [`trim_lines`]. Since those are
/// expected to trim line ends, we only bother with empty strings in this
/// implementation.
struct TrimEndIterator<T: Iterator<Item = String>> {
	inner: T,
	empty_string_run_count: usize,
	next_non_empty_string: Option<String>,
}

impl<T: Iterator<Item = String>> TrimEndIterator<T> {
	pub fn wrap(inner: T) -> Self {
		TrimEndIterator {
			inner,
			empty_string_run_count: 0,
			next_non_empty_string: None,
		}
	}
}

impl<T: Iterator<Item = String>> Iterator for TrimEndIterator<T> {
	type Item = String;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			if self.next_non_empty_string.is_some() {
				// we want to remove only trailing empty strings, so we keep
				// runs of empty strings in the middle intact
				if self.empty_string_run_count > 0 {
					self.empty_string_run_count -= 1;
					return Some(String::new());
				}
				return std::mem::take(&mut self.next_non_empty_string);
			}

			if let Some(next) = self.inner.next() {
				if next.len() > 0 {
					// we can't return this directly because there may have
					// been a run of empty strings before it that we need to
					// return
					self.next_non_empty_string = Some(next);
				} else {
					self.empty_string_run_count += 1;
				}
			} else {
				return None;
			}
		}
	}
}

#[cfg(test)]
mod test_lines {
	use super::lines;

	#[test]
	fn returns_single_line_for_no_line_break() {
		let out = lines("single line");
		assert_eq!(out, vec!["single line"]);
	}

	#[test]
	fn supports_multiple_line_break_sequences() {
		let out = lines("line 1\nline 2\r\nline 3\rline 4");
		assert_eq!(out, vec!["line 1", "line 2", "line 3", "line 4"]);
	}

	#[test]
	fn returns_empty_for_empty_string() {
		let out = lines("");
		assert!(out.len() == 0);
	}

	#[test]
	fn skips_leading_empty_lines() {
		let out = lines("\n\n  \r\n\t\rfirst non-empty\nsecond");
		assert_eq!(out, vec!["first non-empty", "second"]);
	}

	#[test]
	fn removes_trailing_empty_lines() {
		let out = lines("first\nlast non-empty\n\n  \r\n\t\r\n");
		assert_eq!(out, vec!["first", "last non-empty"]);
	}

	#[test]
	fn trim_line_ends() {
		let out = lines("l1  \nl2\t\r\nl3\t \rl4  ");
		assert_eq!(out, vec!["l1", "l2", "l3", "l4"]);
	}

	#[test]
	fn does_not_strip_lead_indentation() {
		let out = lines("\n 1\n  2\n   3");
		assert_eq!(out, vec![" 1", "  2", "   3"]);
	}
}

#[cfg(test)]
mod test_trim_lines {
	use super::trim_lines;

	#[test]
	fn returns_non_empty_lines() {
		let input = vec!["1", "2", "3"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["1", "2", "3"]);
	}

	#[test]
	fn skips_leading_blank_lines() {
		let input = vec!["", "  ", "\t", "first non-empty", "second"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["first non-empty", "second"]);
	}

	#[test]
	fn removes_trailing_space_from_lines() {
		let input = vec!["l1  ", "l2\t", "l3\t ", "l4  "];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["l1", "l2", "l3", "l4"]);
	}

	#[test]
	fn removes_trailing_blank_lines() {
		let input = vec!["first", "last non-empty", "", "  ", "\t"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["first", "last non-empty"]);
	}

	#[test]
	fn does_not_remove_blank_lines_from_the_middle() {
		let input = vec!["1x", "", "2x", "", "", "3x", "", "", "", "!"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec!["1x", "", "2x", "", "", "3x", "", "", "", "!"]);
	}

	#[test]
	fn does_not_trim_lead_indentation() {
		let input = vec![" 1", "  2", "   3"];
		let out = trim_lines(input.iter()).collect::<Vec<_>>();
		assert_eq!(out, vec![" 1", "  2", "   3"]);
	}
}
