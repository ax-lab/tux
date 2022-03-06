pub fn trim<S: AsRef<str>>(input: S) -> String {
	let lines = super::lines(input);
	if lines.len() == 0 {
		return String::new();
	}
	let first = lines[0].as_str();
	let trimmed = first.trim_start();
	let indent_length = first.len() - trimmed.len();
	let indent = &first[..indent_length];
	let lines = lines
		.iter()
		.map(|x| x.strip_prefix(indent).unwrap_or(x.as_str()))
		.collect::<Vec<_>>();
	lines.join("\n")
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn trim_of_empty_string() {
		let out = trim("");
		assert_eq!(out, "");
	}

	#[test]
	fn trim_removes_extra_blank_lines() {
		let out = trim("\n \n  \ntext\n\n \n  \n");
		assert_eq!(out, "text");
	}

	#[test]
	fn trim_removes_first_line_indentation_from_all_lines() {
		let out = trim("\n  l1\n\n    l2\n  l3\n");
		assert_eq!(out, "l1\n\n  l2\nl3");
	}
}
