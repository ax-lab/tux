use std::{collections::VecDeque, io::ErrorKind, path::Path};

const TEST_INPUT_FILE_EXTENSION: &'static str = "input";
const TEST_OUTPUT_FILE_EXTENSION: &'static str = "valid";
const TEST_NEW_OUTPUT_FILE_EXTENSION: &'static str = "valid.new";

/// Test all input files in the given directory (recursively) using the
/// provided callback and compare the expected output.
///
/// A test input is any text file with a `.input` extension. Each input file
/// must have a corresponding `.valid` file providing the expected output.
///
/// For each input file, the test callback is called with the input text split
/// by lines. The result of the callback are the test output lines.
///
/// The test is successful if the callback output matches the `.valid` file
/// contents.
pub fn testdata<P, F>(path: P, callback: F)
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let result = testdata_to_result(path, callback);
	if !result.success() {
		panic!("tests failed");
	}
}

#[derive(Debug)]
struct TestDataResult {
	pub tests: Vec<TestDataResultItem>,
}

impl TestDataResult {
	pub fn success(&self) -> bool {
		for it in self.tests.iter() {
			if !it.success {
				return false;
			}
		}
		true
	}
}

#[derive(Debug)]
struct TestDataResultItem {
	pub success: bool,

	#[allow(dead_code)]
	pub name: String,
}

fn testdata_to_result<P, F>(path: P, mut callback: F) -> TestDataResult
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let root_path = path.as_ref();
	let mut test_input_list = Vec::new();

	let mut dir_queue = VecDeque::new();
	dir_queue.push_back((root_path.to_owned(), String::new()));

	while let Some((next_path, base_name)) = dir_queue.pop_front() {
		let entries = std::fs::read_dir(&next_path).expect("reading test directory");
		let entries = entries.map(|x| x.expect("reading test directory entry"));

		let mut entries = entries.collect::<Vec<_>>();
		entries.sort_by_key(|x| x.file_name());

		for entry in entries {
			let entry_path = entry.path();
			let entry_name = if base_name.len() > 0 {
				format!("{}/{}", base_name, entry.file_name().to_string_lossy())
			} else {
				entry.file_name().to_string_lossy().to_string()
			};

			let entry = std::fs::metadata(&entry_path).expect("reading test directory metadata");
			if entry.is_dir() {
				dir_queue.push_back((entry_path, entry_name));
			} else if let Some(extension) = entry_path.extension() {
				if extension == TEST_INPUT_FILE_EXTENSION {
					test_input_list.push((entry_path, entry_name));
				}
			}
		}
	}

	let mut success = true;
	let mut tests = Vec::new();

	for (path, name) in test_input_list.into_iter() {
		let input = std::fs::read_to_string(&path).expect("reading test input file");
		let input = input.trim_end();
		let input = input
			.lines()
			.skip_while(|x| x.trim() == "")
			.map(|x| x.to_string())
			.collect();

		let mut test_success = true;
		let output = callback(input);
		let output = output.join("\n").trim().to_string();

		let mut output_path = path.clone();
		output_path.set_extension(TEST_OUTPUT_FILE_EXTENSION);
		match std::fs::read_to_string(&output_path) {
			Ok(expected) => {
				let expected = expected
					.lines()
					.skip_while(|x| x.trim() == "")
					.collect::<Vec<_>>()
					.join("\n");
				let expected = expected.trim_end();
				if output != expected {
					test_success = false;
				}
			}
			Err(err) => {
				test_success = false;
				if err.kind() == ErrorKind::NotFound {
					// for convenience, if the test output is not found we
					// generate a new one with the current test output
					let mut output_path = output_path.clone();
					output_path.set_extension(TEST_NEW_OUTPUT_FILE_EXTENSION);
					std::fs::write(output_path, output).expect("writing new test output");
				} else {
					panic!("failed to read output file for {}: {}", name, err);
				}
			}
		}

		success = success && test_success;
		tests.push(TestDataResultItem {
			success: test_success,
			name: name,
		})
	}

	TestDataResult { tests }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::temp::TestTempDir;

	#[test]
	fn testdata_runs_test_callback() {
		let dir = TestTempDir::create_new();
		dir.create_file("some.input", "");
		dir.create_file("some.valid", "");

		let mut test_callback_was_called = false;
		testdata(dir.path(), |input| {
			test_callback_was_called = true;
			input
		});

		assert!(test_callback_was_called);
	}

	#[test]
	fn testdata_runs_test_callback_with_input() {
		let dir = TestTempDir::create_new();
		dir.create_file("some.input", "the input");
		dir.create_file("some.valid", "");

		let mut test_callback_input = String::new();
		testdata(dir.path(), |input| {
			let input = input.join("\n");
			test_callback_input.push_str(&input);
			Vec::new()
		});

		assert_eq!(test_callback_input, "the input");
	}

	#[test]
	#[should_panic]
	fn testdata_fails_if_output_is_missing() {
		let dir = TestTempDir::create_new();
		dir.create_file("test.input", "some input");

		testdata(dir.path(), |input| input);
	}

	#[test]
	#[should_panic]
	fn testdata_fails_if_output_is_different() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "test.input", "some input", "some output");
		testdata(dir.path(), |input| input);
	}

	#[test]
	fn testdata_ignores_leading_and_trailing_lines_when_comparing() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "test.input", "value", "  \n\nvalue\n\n  ");
		testdata(dir.path(), |input| input);

		testdata(dir.path(), |mut input| {
			input.push("".to_string());
			input
		});
	}

	#[test]
	fn testdata_ignores_line_break_differences_in_input_and_output() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "a\nb\nc", "c\r\nb\r\na");
		helper::write_case(&dir, "b.input", "a\r\nb\r\nc", "c\nb\na");

		testdata(dir.path(), |mut input| {
			input.reverse();
			input
		});
	}

	#[test]
	#[should_panic]
	fn testdata_should_not_ignore_trailing_indentation_of_first_line() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "test.input", "value", "  value");
		testdata(dir.path(), |input| input);

		testdata(dir.path(), |mut input| {
			input.push("".to_string());
			input
		});
	}

	#[test]
	fn testdata_trim_leading_empty_lines_and_trailing_space_in_the_input() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "\n  \n  a\n\nb\nc  \n  \n\n", "");

		let mut test_input = String::new();
		testdata(dir.path(), |input| {
			test_input = input.join("\n");
			Vec::new()
		});

		assert_eq!(test_input, "  a\n\nb\nc");
	}

	#[test]
	fn testdata_runs_test_callback_for_each_input() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "input A", "");
		helper::write_case(&dir, "b.input", "input B", "");
		helper::write_case(&dir, "c.input", "input C", "");

		let mut test_callback_inputs = Vec::new();
		testdata(dir.path(), |input| {
			let input = input.join("\n");
			test_callback_inputs.push(input);
			Vec::new()
		});

		let expected = vec![
			"input A".to_string(),
			"input B".to_string(),
			"input C".to_string(),
		];
		assert_eq!(test_callback_inputs, expected);
	}

	#[test]
	fn testdata_checks_subdirectories() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a1.input", "a1", "");
		helper::write_case(&dir, "a2.input", "a2", "");
		helper::write_case(&dir, "a3.input", "a3", "");
		helper::write_case(&dir, "a1/a.input", "a1/a", "");
		helper::write_case(&dir, "a1/b.input", "a1/b", "");
		helper::write_case(&dir, "a2/a.input", "a2/a", "");
		helper::write_case(&dir, "a2/b.input", "a2/b", "");
		helper::write_case(&dir, "a2/sub/file.input", "a2/sub/file", "");

		let mut test_callback_inputs = Vec::new();
		testdata(dir.path(), |input| {
			let input = input.join("\n");
			test_callback_inputs.push(input);
			Vec::new()
		});

		let expected = vec![
			"a1".to_string(),
			"a2".to_string(),
			"a3".to_string(),
			"a1/a".to_string(),
			"a1/b".to_string(),
			"a2/a".to_string(),
			"a2/b".to_string(),
			"a2/sub/file".to_string(),
		];
		assert_eq!(test_callback_inputs, expected);
	}

	#[test]
	fn testdata_to_result_returns_ok_for_valid_case() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "test.input", "abc\n123", "123\nabc");

		let result = testdata_to_result(dir.path(), |mut input| {
			input.reverse();
			input
		});

		assert!(result.success());
		assert_eq!(result.tests.len(), 1);
		assert_eq!(result.tests[0].name, "test.input");
		assert_eq!(result.tests[0].success, true);
	}

	#[test]
	fn testdata_to_result_returns_an_item_for_each_case() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "A", "a");
		helper::write_case(&dir, "b.input", "B", "b");
		helper::write_case(&dir, "sub/some.input", "Some", "some");

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});

		assert_eq!(result.tests.len(), 3);
		assert_eq!(result.tests[0].name, "a.input");
		assert_eq!(result.tests[1].name, "b.input");
		assert_eq!(result.tests[2].name, "sub/some.input");
	}

	#[test]
	fn testdata_should_fail_and_generate_an_output_file_if_one_does_not_exist() {
		let dir = TestTempDir::create_new();
		dir.create_file("test.input", "Some Input");

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});
		assert_eq!(result.success(), false);

		let new_result_path = dir.path().join("test.valid.new");
		assert!(new_result_path.is_file());

		let new_result_text = std::fs::read_to_string(new_result_path).unwrap();
		assert_eq!(new_result_text, "some input");
	}

	#[test]
	fn testdata_to_result_should_fail_if_output_does_not_match() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "Valid 1", "valid 1");
		helper::write_case(&dir, "b.input", "Valid 2", "valid 2");
		helper::write_case(
			&dir,
			"c.input",
			"this should fail",
			"invalid output for the test",
		);

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});

		assert!(!result.success());
		assert_eq!(result.tests.len(), 3);
		assert!(result.tests[0].success);
		assert!(result.tests[1].success);
		assert!(!result.tests[2].success);
	}

	mod helper {
		use super::*;

		pub fn write_case(dir: &TestTempDir, input_file: &str, input: &str, expected: &str) {
			dir.create_file(input_file, input);

			let suffix = format!(".{}", TEST_INPUT_FILE_EXTENSION);
			let basename = input_file.strip_suffix(&suffix).unwrap();
			dir.create_file(
				&format!("{}.{}", basename, TEST_OUTPUT_FILE_EXTENSION),
				expected,
			);
		}
	}
}
