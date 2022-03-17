/// Single operation in a diff between two lists of items. A complete diff
/// consists of a sequence of these operations.
///
/// Using these operations and items from both lists it is possible to
/// generate the diff representation.
///
/// Each operation applies to a number of items from either list. Operations
/// in the diff must be considered in sequence, with each operation moving
/// forward in the respective list (or both lists).
#[derive(Debug)]
pub enum Diff {
	/// Output a sequence of items that are common between the `source`
	/// and `result` lists. Moves forward in both lists.
	Output(usize),

	/// Delete a sequence of items from the `source` list, moving forward
	/// in the list.
	Delete(usize),

	/// Insert a sequence of items from the `result` list, moving forward
	/// in the list.
	Insert(usize),
}

/// Computes the difference between two lists containing lines of text.
///
/// Returns a vector of [`Diff`] entries from `source` to `result`. If
/// the lists are equal, the returned vector is empty.
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
		diff.push(Diff::Output(common_prefix));
	}

	let mut cur_source = 0;
	let mut cur_result = 0;

	for (line_source, line_result) in common_subsequence {
		if line_source > cur_source {
			diff.push(Diff::Delete(line_source - cur_source));
		}

		if line_result > cur_result {
			diff.push(Diff::Insert(line_result - cur_result));
		}

		match diff.last_mut() {
			Some(Diff::Output(count)) => {
				*count += 1;
			}
			_ => {
				diff.push(Diff::Output(1));
			}
		}

		cur_source = line_source + 1;
		cur_result = line_result + 1;
	}

	if cur_source < source.len() {
		diff.push(Diff::Delete(source.len() - cur_source));
	}

	if cur_result < result.len() {
		diff.push(Diff::Insert(result.len() - cur_result));
	}

	if common_suffix > 0 {
		diff.push(Diff::Output(common_suffix));
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
			let mut offset_source = 0;
			let mut offset_result = 0;
			let output = diff
				.into_iter()
				.flat_map(|item| -> Vec<String> {
					match item {
						Diff::Output(count) => {
							let sta = offset_source;
							let end = sta + count;
							offset_source += count;
							offset_result += count;
							(sta..end).map(|x| format!(" {}", source[x])).collect()
						}
						Diff::Insert(count) => {
							let sta = offset_result;
							let end = sta + count;
							offset_result += count;
							(sta..end).map(|x| format!("+{}", result[x])).collect()
						}
						Diff::Delete(count) => {
							let sta = offset_source;
							let end = sta + count;
							offset_source += count;
							(sta..end).map(|x| format!("-{}", source[x])).collect()
						}
					}
				})
				.collect::<Vec<String>>();
			output.join("\n")
		}

		fn sanity_check_diff(diff: Vec<Diff>) -> Vec<Diff> {
			check_diff_does_not_have_contiguous_output_ranges(&diff);
			check_diff_does_not_have_empty_entries(&diff);
			diff
		}

		fn check_diff_does_not_have_empty_entries(diff: &Vec<Diff>) {
			for it in diff.iter() {
				let is_empty = match it {
					Diff::Output(size) | Diff::Insert(size) | Diff::Delete(size) => *size == 0,
				};
				if is_empty {
					panic!("diff produced empty entries:\n{:?}", diff);
				}
			}
		}

		fn check_diff_does_not_have_contiguous_output_ranges(diff: &Vec<Diff>) {
			let mut last_was_output = false;
			let contiguous_output_ranges = diff.iter().filter(|x| {
				if let Diff::Output(_) = x {
					let is_contiguous = last_was_output;
					last_was_output = true;
					is_contiguous
				} else {
					last_was_output = false;
					false
				}
			});

			if contiguous_output_ranges.count() > 0 {
				panic!("diff produced contiguous output ranges:\n{:?}", diff);
			}
		}
	}
}
