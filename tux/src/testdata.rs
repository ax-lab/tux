use std::{
	collections::VecDeque,
	io::ErrorKind,
	path::{Path, PathBuf},
};

const TEST_INPUT_FILE_EXTENSION: &'static str = "input";
const TEST_VALID_FILE_EXTENSION: &'static str = "valid";
const TEST_NEW_VALID_FILE_EXTENSION: &'static str = "valid.new";

/// Test all input files in the given directory (recursively) using the
/// provided callback and comparing the expected output.
///
/// ## Test files
///
/// Any files with the `.input` extension will be loaded and the lines passed
/// to the test callback. The callback result is then compared to the expected
/// output.
///
/// The expected output for the test callback must be provided in a `.valid`
/// file with the same name as the `.input`.
///
/// ## Test condition
///
/// The test will fail if the callback output does not match the `.valid.`
/// file. In that case, the test will output a diff between the actual
/// and the expected output.
///
/// ## Generating valid files
///
/// As a convenience feature, if a `.valid` file is not found, the test will
/// fail but will create a `.valid.new` file with the actual output.
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

	if !result.success() {
		let mut failed_count = 0;

		for it in result.tests.iter() {
			if !it.success {
				failed_count += 1;

				if let Some(expected) = &it.expect {
					eprintln!(
						"\n=> `{}` output did not match `{}`:",
						it.name, it.valid_file
					);

					let diff = super::diff::lines(&it.actual, expected);
					eprintln!("\n{}", diff);
				} else {
					eprintln!("\n=> `{}` for test `{}` not found", it.valid_file, it.name);
					eprintln!(
						".. created `{}.new` with the current test output",
						it.valid_file
					);
				}
			}
		}

		eprintln!("\n===== Failed tests =====\n");
		for it in result.tests.iter() {
			if !it.success {
				eprintln!("- {}", it.name);
			}
		}
		eprintln!();

		panic!(
			"{} test case{} failed",
			failed_count,
			if failed_count != 1 { "s" } else { "" }
		);
	}
}

#[derive(Debug)]
struct TestDataResult {
	pub tests: Vec<TestDataResultItem>,
}

/// Contains information about a single test run (equivalent to a single input
/// file).
#[derive(Debug)]
struct TestDataResultItem {
	/// Returns if the test was successful.
	pub success: bool,

	/// The test name. This is the input file name, without path.
	pub name: String,

	/// Name for the valid file containing the expected test output.
	pub valid_file: String,

	/// Expected test output from the valid file. This will be `None` if the
	/// test failed because the valid file was not found.
	pub expect: Option<Vec<String>>,

	/// Actual output form the test callback.
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

fn testdata_to_result<P, F>(test_path: P, mut test_callback: F) -> TestDataResult
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let test_path = test_path.as_ref();

	let mut test_results = Vec::new();
	let test_inputs_with_name = collect_test_inputs_with_name(test_path);

	for (input_path, test_name) in test_inputs_with_name.into_iter() {
		let input_text = std::fs::read_to_string(&input_path).expect("reading test input file");
		let input_lines = super::text::lines(input_text);

		let mut test_succeeded = true;
		let output_lines = test_callback(input_lines);
		let output_text = output_lines.join("\n");

		let mut valid_file_path = input_path.clone();
		valid_file_path.set_extension(TEST_VALID_FILE_EXTENSION);

		let expected_lines = match std::fs::read_to_string(&valid_file_path) {
			Ok(raw_text) => {
				let expected_lines = super::text::lines(raw_text);
				let expected_text = expected_lines.join("\n");
				if output_text != expected_text {
					test_succeeded = false;
				}
				Some(expected_lines)
			}
			Err(err) => {
				test_succeeded = false;
				if err.kind() == ErrorKind::NotFound {
					// for convenience, if the test output is not found we
					// generate a new one with the current test output
					let mut new_valid_file_path = valid_file_path.clone();
					new_valid_file_path.set_extension(TEST_NEW_VALID_FILE_EXTENSION);
					std::fs::write(new_valid_file_path, output_text)
						.expect("writing new test output");
				} else {
					// this is not an expected failure mode, so we just panic
					panic!("failed to read output file for {}: {}", test_name, err);
				}

				// there is no expected lines in this case, since the valid
				// file was not found
				None
			}
		};

		let valid_file_name = valid_file_path.file_name().unwrap().to_string_lossy();
		test_results.push(TestDataResultItem {
			success: test_succeeded,
			name: test_name,
			valid_file: valid_file_name.into(),
			expect: expected_lines,
			actual: output_lines,
		});
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
	use crate::{temp_dir, TempDir};

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

		pub fn write_case(dir: &TempDir, input_file: &str, input: &str, expected: &str) {
			dir.create_file(input_file, input);

			let suffix = format!(".{}", TEST_INPUT_FILE_EXTENSION);
			let basename = input_file.strip_suffix(&suffix).unwrap();
			dir.create_file(
				&format!("{}.{}", basename, TEST_VALID_FILE_EXTENSION),
				expected,
			);
		}
	}
}
