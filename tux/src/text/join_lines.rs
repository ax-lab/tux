/// Join lines from a string iterator into a single string.
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
mod tests {
	use super::*;

	#[test]
	fn join_empty_slice_returns_empty() {
		let out = join_lines(Vec::<String>::new());
		assert_eq!(out, "");
	}

	#[test]
	fn join_returns_lines_joined_by_line_break() {
		let out = join_lines(["a", "b", "c"]);
		assert_eq!(out, "a\nb\nc");
	}
}
