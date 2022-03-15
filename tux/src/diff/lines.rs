/// Entry for a diff between two lists of items. Each entry represents an edit
/// operation that applies to the source list to reach the result list.
#[derive(Debug)]
pub enum Diff {
	/// Output a range from the source list that is also present in the
	/// result list.
	///
	/// The range is given by the indexes `(start, end)` from the `source`
	/// list, where `end` is exclusive.
	Output(usize, usize),

	/// Delete a range from the source list.
	///
	/// The range is given by the indexes `(start, end)` from the `source`
	/// list, where `end` is exclusive.
	Delete(usize, usize),

	/// Insert a range from the result list into the source list.
	///
	/// The range is given by the indexes `(start, end)` from the `result`
	/// list.
	///
	/// The insertion point in the `source` list is always at the end of
	/// the previous [`Diff::Output`] or [`Diff::Insert`] entry in the diff.
	Insert(usize, usize),
}

/// Computes the line difference from `source` to `result`.
///
/// Returns a list of [`Diff`] entries. If `source` and `result` are equal,
/// the returned list is empty.
pub fn lines<T>(source: &[T], result: &[T]) -> Vec<Diff>
where
	T: AsRef<str> + std::cmp::PartialEq,
{
	let common_prefix = {
		let mut len = 0;
		while len < source.len() && len < result.len() && source[len] == result[len] {
			len += 1;
		}
		len
	};

	let source = &source[common_prefix..];
	let result = &result[common_prefix..];

	let no_difference = source.len() == 0 && result.len() == 0;
	if no_difference {
		return Vec::new();
	}

	let common_suffix = {
		let from_last = |slice: &[T], offset| slice.len() - offset - 1;
		let mut len = 0;
		while len < source.len()
			&& len < result.len()
			&& source[from_last(source, len)] == result[from_last(result, len)]
		{
			len += 1;
		}
		len
	};

	let source = &source[..source.len() - common_suffix];
	let result = &result[..result.len() - common_suffix];

	let common_subsequence = super::lcs(source, result);

	let mut diff = Vec::new();
	if common_prefix > 0 {
		diff.push(Diff::Output(0, common_prefix));
	}

	let mut cur_source = 0;
	let mut cur_result = 0;

	let offset = common_prefix;
	for (line_source, line_result) in common_subsequence {
		if line_source > cur_source {
			diff.push(Diff::Delete(cur_source + offset, line_source + offset));
		}

		if line_result > cur_result {
			diff.push(Diff::Insert(cur_result + offset, line_result + offset));
		}

		let current_line = line_source + offset;
		match diff.last_mut() {
			Some(Diff::Output(_, end)) if *end == current_line => {
				*end = current_line + 1;
			}
			_ => {
				diff.push(Diff::Output(current_line, current_line + 1));
			}
		}

		cur_source = line_source + 1;
		cur_result = line_result + 1;
	}

	if cur_source < source.len() {
		diff.push(Diff::Delete(cur_source + offset, source.len() + offset));
	}

	if cur_result < result.len() {
		diff.push(Diff::Insert(cur_result + offset, result.len() + offset));
	}

	if common_suffix > 0 {
		diff.push(Diff::Output(
			source.len() + offset,
			source.len() + offset + common_suffix,
		));
	}

	diff
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::text;

	#[test]
	fn diff_lines_empty() {
		let a: Vec<String> = Vec::new();
		let b: Vec<String> = Vec::new();
		let diff = lines(&a, &b);
		assert!(diff.len() == 0);
	}

	#[test]
	fn diff_lines_equal() {
		let a = vec!["line 1", "line 2"];
		let b = vec!["line 1", "line 2"];
		let diff = lines(&a, &b);
		assert!(diff.len() == 0);
	}

	#[test]
	fn diff_lines_empty_result() {
		let a = vec!["line 1", "line 2"];
		let b = Vec::new();
		let diff = helper::diff_to_text(a, b);
		assert_eq!(diff, text::join_lines(["-line 1", "-line 2"]));
	}

	#[test]
	fn diff_lines_empty_source() {
		let a = Vec::new();
		let b = vec!["line 1", "line 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(diff, text::join_lines(["+line 1", "+line 2"]));
	}

	#[test]
	fn diff_lines_nothing_in_common() {
		let a = vec!["line 1", "line 2"];
		let b = vec!["line A", "line B"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-line 1", "-line 2", "+line A", "+line B"])
		);
	}

	#[test]
	fn diff_lines_removed_suffix() {
		let a = vec!["same 1", "same 2", "suffix 1", "suffix 2"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([" same 1", " same 2", "-suffix 1", "-suffix 2"])
		);
	}

	#[test]
	fn diff_lines_added_suffix() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["same 1", "same 2", "suffix 1", "suffix 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([" same 1", " same 2", "+suffix 1", "+suffix 2"])
		);
	}

	#[test]
	fn diff_lines_removed_prefix() {
		let a = vec!["prefix 1", "prefix 2", "same 1", "same 2"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-prefix 1", "-prefix 2", " same 1", " same 2"])
		);
	}

	#[test]
	fn diff_lines_added_prefix() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["prefix 1", "prefix 2", "same 1", "same 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["+prefix 1", "+prefix 2", " same 1", " same 2"])
		);
	}

	#[test]
	fn diff_lines_removed_prefix_and_suffix() {
		let a = vec![
			"prefix 1", "prefix 2", "same 1", "same 2", "suffix 1", "suffix 2",
		];
		let b = vec!["same 1", "same 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([
				"-prefix 1",
				"-prefix 2",
				" same 1",
				" same 2",
				"-suffix 1",
				"-suffix 2"
			])
		);
	}

	#[test]
	fn diff_lines_added_prefix_and_suffix() {
		let a = vec!["same 1", "same 2"];
		let b = vec![
			"prefix 1", "prefix 2", "same 1", "same 2", "suffix 1", "suffix 2",
		];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([
				"+prefix 1",
				"+prefix 2",
				" same 1",
				" same 2",
				"+suffix 1",
				"+suffix 2"
			])
		);
	}

	#[test]
	fn diff_lines_added_with_two_common_lines() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["prefix", "same 1", "between", "same 2", "suffix"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["+prefix", " same 1", "+between", " same 2", "+suffix"])
		);
	}

	#[test]
	fn diff_lines_removed_with_two_common_lines() {
		let a = vec!["prefix", "same 1", "between", "same 2", "suffix"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-prefix", " same 1", "-between", " same 2", "-suffix"])
		);
	}

	#[test]
	fn diff_lines_with_different_contents_with_two_common_lines() {
		let a = vec!["prefix A", "same 1", "between A", "same 2", "suffix A"];
		let b = vec!["prefix B", "same 1", "between B", "same 2", "suffix B"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([
				"-prefix A",
				"+prefix B",
				" same 1",
				"-between A",
				"+between B",
				" same 2",
				"-suffix A",
				"+suffix B"
			])
		);
	}

	#[test]
	fn diff_lines_with_non_trivial_common_sequence() {
		let a = vec!["a1", "sX", "a2", "sW", "sX", "a3", "sY", "a4", "sZ"];
		let b = vec!["b1", "b2", "sW", "sX", "b3", "sY", "b4", "sZ"];
		let diff = helper::diff_to_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([
				"-a1", "-sX", "-a2", "+b1", "+b2", " sW", " sX", "-a3", "+b3", " sY", "-a4", "+b4",
				" sZ",
			])
		);
	}

	mod helper {
		use super::*;

		pub fn diff_to_text(a: Vec<&str>, b: Vec<&str>) -> String {
			let diff = lines(&a, &b);
			let diff = sanity_check_diff(diff);
			diff_to_string(diff, &a, &b)
		}

		fn diff_to_string(diff: Vec<Diff>, source: &Vec<&str>, result: &Vec<&str>) -> String {
			let output = diff
				.into_iter()
				.flat_map(|item| -> Vec<String> {
					match item {
						Diff::Output(sta, end) => {
							(sta..end).map(|x| format!(" {}", source[x])).collect()
						}
						Diff::Insert(sta, end) => {
							(sta..end).map(|x| format!("+{}", result[x])).collect()
						}
						Diff::Delete(sta, end) => {
							(sta..end).map(|x| format!("-{}", source[x])).collect()
						}
					}
				})
				.collect::<Vec<String>>();
			output.join("\n")
		}

		fn sanity_check_diff(diff: Vec<Diff>) -> Vec<Diff> {
			check_diff_does_not_have_contiguous_output_ranges(&diff);
			diff
		}

		fn check_diff_does_not_have_contiguous_output_ranges(diff: &Vec<Diff>) {
			let all_output_ranges = diff.iter().map(|x| {
				if let &Diff::Output(a, b) = x {
					Some((a, b))
				} else {
					None
				}
			});

			let mut last_range_end = None;
			let contiguous_output_ranges = all_output_ranges.filter(|x| {
				if let &Some((start, end)) = x {
					let is_contiguous = if let Some(last_range_end) = last_range_end {
						start == last_range_end
					} else {
						false
					};
					last_range_end = Some(end);
					is_contiguous
				} else {
					last_range_end = None;
					false
				}
			});

			if contiguous_output_ranges.count() > 0 {
				panic!("diff produced contiguous output ranges:\n{:?}", diff);
			}
		}
	}
}
