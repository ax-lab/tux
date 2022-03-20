/// Joins a sequence of strings into a single string separate by line breaks.
pub fn join_lines<T>(lines: T) -> String
where
	T: IntoIterator,
	T::Item: AsRef<str>,
{
	let lines = lines.into_iter().collect::<Vec<_>>();
	lines
		.iter()
		.map(|x| x.as_ref())
		.collect::<Vec<_>>()
		.join("\n")
}

#[cfg(test)]
mod test_join_lines {
	use super::join_lines;

	#[test]
	fn with_empty_sequence_returns_empty() {
		let out = join_lines(Vec::<String>::new());
		assert_eq!(out, "");
	}

	#[test]
	fn returns_lines_joined_by_line_break() {
		let out = join_lines(["a", "b", "c"]);
		assert_eq!(out, "a\nb\nc");
	}
}
