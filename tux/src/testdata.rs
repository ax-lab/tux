use std::{
	collections::VecDeque,
	io::ErrorKind,
	path::{Path, PathBuf},
};

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

	for it in result.tests.iter() {
		if it.success {
			println!("passed: {}", it.name);
		} else {
			println!("failed: {}", it.name);
		}
	}

	for it in result.tests.iter() {
		if !it.success {
			eprintln!(
				"\n=> `{}` output did not match `{}`:\n",
				it.name, it.valid_file
			);
			let diff = super::diff::lines(&it.actual, &it.expect);
			eprintln!("{}", diff);
		}
	}

	if !result.success() {
		eprintln!("\n===== Failed tests =====\n");
		for it in result.tests.iter() {
			if !it.success {
				eprintln!("- {}", it.name);
			}
		}
		eprintln!();

		panic!("one or more tests failed");
	}
}

#[derive(Debug)]
struct TestDataResult {
	pub tests: Vec<TestDataResultItem>,
}

#[derive(Debug)]
struct TestDataResultItem {
	pub success: bool,
	pub name: String,
	pub valid_file: String,
	pub expect: Vec<String>,
	pub actual: Vec<String>,
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

fn testdata_to_result<P, F>(root_path: P, mut test_callback: F) -> TestDataResult
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let root_path = root_path.as_ref();

	let mut test_results = Vec::new();
	let test_inputs_with_name = collect_test_inputs_with_name(root_path);

	for (input_path, test_name) in test_inputs_with_name.into_iter() {
		let input = std::fs::read_to_string(&input_path).expect("reading test input file");
		let input = super::text::lines(input);

		let mut test_succeeded = true;
		let output_lines = test_callback(input);
		let output = output_lines.join("\n");

		let mut output_path = input_path.clone();
		output_path.set_extension(TEST_OUTPUT_FILE_EXTENSION);
		let expected_output = match std::fs::read_to_string(&output_path) {
			Ok(expected_output) => {
				let expected_lines = super::text::lines(expected_output);
				let expected_output = expected_lines.join("\n");
				if output != expected_output {
					test_succeeded = false;
				}
				expected_lines
			}
			Err(err) => {
				test_succeeded = false;
				if err.kind() == ErrorKind::NotFound {
					// for convenience, if the test output is not found we
					// generate a new one with the current test output
					let mut output_path = output_path.clone();
					output_path.set_extension(TEST_NEW_OUTPUT_FILE_EXTENSION);
					std::fs::write(output_path, output).expect("writing new test output");
				} else {
					panic!("failed to read output file for {}: {}", test_name, err);
				}
				Vec::new()
			}
		};

		test_results.push(TestDataResultItem {
			success: test_succeeded,
			name: test_name,
			valid_file: output_path.file_name().unwrap().to_string_lossy().into(),
			expect: expected_output,
			actual: output_lines,
		})
	}

	TestDataResult {
		tests: test_results,
	}
}

fn collect_test_inputs_with_name(root_path: &Path) -> Vec<(PathBuf, String)> {
	let mut test_inputs_with_name = Vec::new();

	let mut dirs_to_scan_with_name = VecDeque::new();
	dirs_to_scan_with_name.push_back((root_path.to_owned(), String::new()));

	while let Some((current_dir, current_name)) = dirs_to_scan_with_name.pop_front() {
		let entries = std::fs::read_dir(&current_dir).expect("reading test directory");
		let entries = entries.map(|x| x.expect("reading test directory entry"));

		// the order here is important to keep the sort order for tests
		let mut entries = entries.collect::<Vec<_>>();
		entries.sort_by_key(|x| x.file_name());

		for entry in entries {
			let entry_path = entry.path();
			let entry_name = if current_name.len() > 0 {
				format!("{}/{}", current_name, entry.file_name().to_string_lossy())
			} else {
				entry.file_name().to_string_lossy().to_string()
			};

			let entry_info =
				std::fs::metadata(&entry_path).expect("reading test directory metadata");
			if entry_info.is_dir() {
				dirs_to_scan_with_name.push_back((entry_path, entry_name));
			} else if let Some(extension) = entry_path.extension() {
				if extension == TEST_INPUT_FILE_EXTENSION {
					test_inputs_with_name.push((entry_path, entry_name));
				}
			}
		}
	}

	test_inputs_with_name
}

//============================================================================//
// Unit tests
//============================================================================//

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{temp_dir, TestTempDir};

	//------------------------------------------------------------------------//
	// Basic tests
	//------------------------------------------------------------------------//

	#[test]
	fn testdata_runs_test_callback() {
		let dir = temp_dir();
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
		let dir = temp_dir();
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
	fn testdata_fails_if_output_is_missing() {
		let dir = temp_dir();
		dir.create_file("test.input", "some input");

		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	#[test]
	fn testdata_fails_if_output_is_different() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "some input", "some output");

		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	#[test]
	fn testdata_runs_test_callback_for_each_input() {
		let dir = temp_dir();
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
	fn testdata_recurses_into_subdirectories() {
		let dir = temp_dir();
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
	fn testdata_fails_and_generate_an_output_file_if_one_does_not_exist() {
		let dir = temp_dir();
		dir.create_file("test.input", "Some Input");

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});
		assert!(!result.success());

		let new_result_path = dir.path().join("test.valid.new");
		assert!(new_result_path.is_file());

		let new_result_text = std::fs::read_to_string(new_result_path).unwrap();
		assert_eq!(new_result_text, "some input");
	}

	//------------------------------------------------------------------------//
	// Input & output trimming
	//------------------------------------------------------------------------//

	#[test]
	fn testdata_trims_input_files() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "\n\nfirst\ntrim end:  \nlast\n\n", "");

		let mut test_input = Vec::new();
		testdata(dir.path(), |input| {
			test_input = input;
			Vec::new()
		});

		assert_eq!(test_input, vec!["first", "trim end:", "last"]);
	}

	#[test]
	fn testdata_trims_expected_output_files() {
		let dir = temp_dir();
		helper::write_case(
			&dir,
			"test.input",
			"line 1\nline 2\nline 3",
			"\n\nline 1\nline 2  \nline 3\n\n",
		);
		testdata(dir.path(), |input| input);
	}

	#[test]
	fn testdata_ignores_line_break_differences_in_input_and_output() {
		let dir = temp_dir();
		helper::write_case(&dir, "a.input", "a\nb\nc", "c\r\nb\r\na");
		helper::write_case(&dir, "b.input", "a\r\nb\r\nc", "c\nb\na");

		testdata(dir.path(), |mut input| {
			input.reverse();
			input
		});
	}

	#[test]
	fn testdata_does_not_ignore_trailing_indentation_of_first_line() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "value", "  value");
		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	//------------------------------------------------------------------------//
	// TestDataResult
	//------------------------------------------------------------------------//

	#[test]
	fn testdata_to_result_returns_ok_for_valid_case() {
		let dir = temp_dir();
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
		let dir = temp_dir();
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
	fn testdata_to_result_fails_if_output_does_not_match() {
		let dir = temp_dir();
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
		assert!(result.tests.len() == 3);
		assert!(result.tests[0].success);
		assert!(result.tests[1].success);
		assert!(!result.tests[2].success);
	}

	//------------------------------------------------------------------------//
	// Helper code
	//------------------------------------------------------------------------//

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
