#[derive(Debug)]
pub enum Diff {
	Output(usize, usize),
	Insert(usize, usize),
	Delete(usize, usize),
}

pub fn diff_lines<A, B>(source: &[A], result: &[B]) -> Vec<Diff>
where
	A: AsRef<str>,
	B: AsRef<str>,
{
	let first_difference = source
		.iter()
		.enumerate()
		.find(|&(n, x)| n >= result.len() || result[n].as_ref() != x.as_ref())
		.map(|x| x.0)
		.unwrap_or(source.len());

	// if source and result are equal we return an empty as a special case
	if first_difference == source.len() && source.len() == result.len() {
		return Vec::new();
	}

	let last_difference = source
		.iter()
		.skip(first_difference)
		.rev()
		.enumerate()
		.find(|&(n, x)| n >= result.len() || result[result.len() - n - 1].as_ref() != x.as_ref())
		.map(|x| source.len() - x.0)
		.unwrap_or(first_difference);

	let mut diff = Vec::new();
	if first_difference > 0 {
		diff.push(Diff::Output(0, first_difference));
	}

	if first_difference < last_difference {
		diff.push(Diff::Delete(first_difference, last_difference));
	}

	let last_difference_in_result = result.len() - (source.len() - last_difference);

	if first_difference < last_difference_in_result {
		diff.push(Diff::Insert(first_difference, last_difference_in_result));
	}

	if last_difference < source.len() {
		diff.push(Diff::Output(last_difference, source.len()));
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
		let diff = diff_lines(&a, &b);
		assert!(diff.len() == 0);
	}

	#[test]
	fn diff_lines_equal() {
		let a = vec!["line 1", "line 2"];
		let b = vec!["line 1", "line 2"];
		let diff = diff_lines(&a, &b);
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

	mod helper {
		use super::*;

		pub fn diff_to_text(a: Vec<&str>, b: Vec<&str>) -> String {
			let diff = diff_lines(&a, &b);
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
