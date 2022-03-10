#[derive(Debug)]
pub enum Diff {
	Output(usize, usize),
	Insert(usize, usize),
	Delete(usize, usize),
}

pub fn lines<A, B>(source: &[A], result: &[B]) -> Vec<Diff>
where
	A: AsRef<str>,
	B: AsRef<str>,
{
	let mut common_start = None;
	'out: for i in 0..source.len() {
		for j in 0..result.len() {
			if source[i].as_ref() == result[j].as_ref() {
				common_start = Some((i, j));
				break 'out;
			}
		}
	}

	let mut diff = Vec::new();

	if let Some((source_sta, result_sta)) = common_start {
		let mut source_end = source_sta + 1;
		let mut result_end = result_sta + 1;
		while true
			&& source_end < source.len()
			&& result_end < result.len()
			&& source[source_end].as_ref() == result[result_end].as_ref()
		{
			source_end += 1;
			result_end += 1;
		}

		if source_sta > 0 {
			diff.push(Diff::Delete(0, source_sta));
		}
		if result_sta > 0 {
			diff.push(Diff::Insert(0, result_sta));
		}

		let are_equal =
			source_sta == 0 && source_end == source.len() && source.len() == result.len();
		if !are_equal {
			diff.push(Diff::Output(source_sta, source_end));
		}

		if source_end < source.len() {
			diff.push(Diff::Delete(source_end, source.len()));
		}
		if result_end < result.len() {
			diff.push(Diff::Insert(result_end, result.len()));
		}
	} else {
		if source.len() > 0 {
			diff.push(Diff::Delete(0, source.len()));
		}
		if result.len() > 0 {
			diff.push(Diff::Insert(0, result.len()));
		}
	};

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

	mod helper {
		use super::*;

		pub fn diff_to_text(a: Vec<&str>, b: Vec<&str>) -> String {
			let diff = lines(&a, &b);
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
	}
}
