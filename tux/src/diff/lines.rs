/// Single element of a diff between a `source` and `result` lists. A complete
/// diff consists of a sequence of these elements (see [`DiffResult`]).
///
/// The sequence of [`Diff`] elements describes the differences between both
/// lists in terms of what happens for each item in them, starting with
/// the `source` list to reach the given `result`. This representation direct
/// translates to common textual representations of a diff.
///
/// Each [`Diff`] elements contains a `count` of items it applies to. Elements
/// must be considered in order when using the diff result:
///
/// - [`Diff::Output`]: the next `count` items from both lists are the same,
/// that is, the sequence from `source` has not been changed in the `result`.
///
/// - [`Diff::Delete`]: the next `count` items from `source` have been removed
/// and don't appear in the `result`.
///
/// - [`Diff::Insert`]: the next `count` items from `result` have been added
/// to the current position in `source`.
///
/// Note that for a position that has both deleted and inserted items, the
/// result will always have the [`Diff::Delete`] before the [`Diff::Insert`].
#[derive(Debug)]
pub enum Diff {
	/// Represents a sequence of items in `source` and `result` that are the
	/// same.
	Output(usize),

	/// Represents a sequence of items from `source` that has been removed
	/// and does not appear in `result`.
	Delete(usize),

	/// Represents a sequence of items from `result` that have been added
	/// at the current position in `source`.
	Insert(usize),
}

/// Computes the difference between two lists containing lines of text.
///
/// # Example
///
/// ```
/// use tux::diff;
///
/// let source = vec!["1", "2", "3"];
/// let result = vec!["0", "2", "4"];
///
/// let diff = diff::lines(&source, &result);
/// println!("{}", diff);
/// ```
///
/// This will output:
///
/// ```text
/// -1
/// +0
///  2
/// -3
/// +4
/// ```
pub fn lines<'a, T>(source: &'a [T], result: &'a [T]) -> DiffResult<'a, T>
where
	T: AsRef<str> + std::cmp::PartialEq,
{
	let full_source = source;
	let full_result = result;

	let common_prefix_len = {
		let mut len = 0;
		while len < source.len() && len < result.len() && source[len] == result[len] {
			len += 1;
		}
		len
	};

	let source = &source[common_prefix_len..];
	let result = &result[common_prefix_len..];

	let both_are_equal = source.len() == 0 && result.len() == 0;
	if both_are_equal {
		return DiffResult {
			items: Vec::new(),
			source: full_source,
			result: full_result,
		};
	}

	let common_suffix_len = {
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

	let source = &source[..source.len() - common_suffix_len];
	let result = &result[..result.len() - common_suffix_len];

	let common_subsequence = super::lcs(source, result);

	let mut diff = Vec::new();
	if common_prefix_len > 0 {
		diff.push(Diff::Output(common_prefix_len));
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

		// if we already have an output element, append to it
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

	if common_suffix_len > 0 {
		diff.push(Diff::Output(common_suffix_len));
	}

	DiffResult {
		items: diff,
		source: full_source,
		result: full_result,
	}
}

/// Result of a diff operation between a `source` list and a `result` list.
///
/// This struct contains a sequence of [`Diff`] elements, and the `source` and
/// `result` lists that the diff sequence was computed from.
///
/// See the [`Diff`] documentation for more details on how the diff is
/// represented.
///
/// # Diff output
///
/// The result maintains a reference to both the `source` and `result` lists
/// that the diff was computed from. This allows it to generate the textual
/// representation of the diff by implementing [`std::fmt::Display`].
///
/// This requires that the `source`/`result` items implement the [`Display`](std::fmt::Display)
/// trait.
pub struct DiffResult<'a, T> {
	items: Vec<Diff>,
	source: &'a [T],
	result: &'a [T],
}

impl<'a, T> DiffResult<'a, T> {
	pub fn is_empty(&self) -> bool {
		self.items.len() == 0
	}

	pub fn items(&self) -> &Vec<Diff> {
		&self.items
	}
}

impl<'a, T> std::fmt::Display for DiffResult<'a, T>
where
	T: std::fmt::Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let source = self.source;
		let result = self.result;

		let mut has_some_output = false;
		let mut start_new_line = |f: &mut std::fmt::Formatter| -> std::fmt::Result {
			if has_some_output {
				write!(f, "\n")
			} else {
				has_some_output = true;
				Ok(())
			}
		};

		let mut cur_offset_source = 0;
		let mut cur_offset_result = 0;
		for item in &self.items {
			match item {
				Diff::Output(count) => {
					let sta = cur_offset_source;
					let end = sta + count;
					cur_offset_source += count;
					cur_offset_result += count;
					for x in sta..end {
						start_new_line(f)?;
						write!(f, " {}", source[x])?;
					}
				}
				Diff::Insert(count) => {
					let sta = cur_offset_result;
					let end = sta + count;
					cur_offset_result += count;
					for x in sta..end {
						start_new_line(f)?;
						write!(f, "+{}", result[x])?;
					}
				}
				Diff::Delete(count) => {
					let sta = cur_offset_source;
					let end = sta + count;
					cur_offset_source += count;
					for x in sta..end {
						start_new_line(f)?;
						write!(f, "-{}", source[x])?;
					}
				}
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod test_lines {
	use super::lines;
	use super::Diff;
	use crate::text;

	#[test]
	fn both_empty() {
		let a: Vec<String> = Vec::new();
		let b: Vec<String> = Vec::new();
		let diff = lines(&a, &b);
		assert!(diff.is_empty());
	}

	#[test]
	fn both_equal() {
		let a = vec!["line 1", "line 2"];
		let b = vec!["line 1", "line 2"];
		let diff = lines(&a, &b);
		assert!(diff.is_empty());
	}

	#[test]
	fn empty_result() {
		let a = vec!["line 1", "line 2"];
		let b = Vec::new();
		let diff = helper::get_diff_text(a, b);
		assert_eq!(diff, text::join_lines(["-line 1", "-line 2"]));
	}

	#[test]
	fn empty_source() {
		let a = Vec::new();
		let b = vec!["line 1", "line 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(diff, text::join_lines(["+line 1", "+line 2"]));
	}

	#[test]
	fn nothing_in_common() {
		let a = vec!["line 1", "line 2"];
		let b = vec!["line A", "line B"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-line 1", "-line 2", "+line A", "+line B"])
		);
	}

	#[test]
	fn removed_suffix() {
		let a = vec!["same 1", "same 2", "suffix 1", "suffix 2"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([" same 1", " same 2", "-suffix 1", "-suffix 2"])
		);
	}

	#[test]
	fn added_suffix() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["same 1", "same 2", "suffix 1", "suffix 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines([" same 1", " same 2", "+suffix 1", "+suffix 2"])
		);
	}

	#[test]
	fn removed_prefix() {
		let a = vec!["prefix 1", "prefix 2", "same 1", "same 2"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-prefix 1", "-prefix 2", " same 1", " same 2"])
		);
	}

	#[test]
	fn added_prefix() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["prefix 1", "prefix 2", "same 1", "same 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["+prefix 1", "+prefix 2", " same 1", " same 2"])
		);
	}

	#[test]
	fn removed_prefix_and_suffix() {
		let a = vec![
			"prefix 1", "prefix 2", "same 1", "same 2", "suffix 1", "suffix 2",
		];
		let b = vec!["same 1", "same 2"];
		let diff = helper::get_diff_text(a, b);
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
	fn added_prefix_and_suffix() {
		let a = vec!["same 1", "same 2"];
		let b = vec![
			"prefix 1", "prefix 2", "same 1", "same 2", "suffix 1", "suffix 2",
		];
		let diff = helper::get_diff_text(a, b);
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
	fn added_with_two_common_lines() {
		let a = vec!["same 1", "same 2"];
		let b = vec!["prefix", "same 1", "between", "same 2", "suffix"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["+prefix", " same 1", "+between", " same 2", "+suffix"])
		);
	}

	#[test]
	fn removed_with_two_common_lines() {
		let a = vec!["prefix", "same 1", "between", "same 2", "suffix"];
		let b = vec!["same 1", "same 2"];
		let diff = helper::get_diff_text(a, b);
		assert_eq!(
			diff,
			text::join_lines(["-prefix", " same 1", "-between", " same 2", "-suffix"])
		);
	}

	#[test]
	fn with_different_contents_and_two_common_lines() {
		let a = vec!["prefix A", "same 1", "between A", "same 2", "suffix A"];
		let b = vec!["prefix B", "same 1", "between B", "same 2", "suffix B"];
		let diff = helper::get_diff_text(a, b);
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
	fn with_non_trivial_common_sequence() {
		let a = vec!["a1", "sX", "a2", "sW", "sX", "a3", "sY", "a4", "sZ"];
		let b = vec!["b1", "b2", "sW", "sX", "b3", "sY", "b4", "sZ"];
		let diff = helper::get_diff_text(a, b);
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

		pub fn get_diff_text(a: Vec<&str>, b: Vec<&str>) -> String {
			let diff = lines(&a, &b);
			sanity_check_diff(&diff.items());
			diff.to_string()
		}

		fn sanity_check_diff(diff: &Vec<Diff>) {
			check_diff_does_not_have_empty_entries(diff);
			check_diff_does_not_have_contiguous_output_ranges(diff);
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
