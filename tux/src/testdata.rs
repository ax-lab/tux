//! Support for tests based on text files.
//!
//! This module is enabled by the `testdata` feature (enabled by default).

use std::{
	collections::VecDeque,
	io::ErrorKind,
	path::{Path, PathBuf},
};

// Changing any of these extensions requires changing all unit and integration
// tests that use this feature, and the `testdata` tests themselves.
const TEST_INPUT_FILE_EXTENSION: &'static str = "input";
const TEST_VALID_FILE_EXTENSION: &'static str = "valid";
const TEST_NEW_VALID_FILE_EXTENSION: &'static str = "valid.new";

/// Test all `.input` files in the given directory (recursively) using the
/// callback and compare the result with the expected output provided by a
/// `.valid` file alongside the input.
///
/// # Test procedure
///
/// All `.input` files in the provided directory will be read as text, split
/// into lines, and passed to the provided callback.
///
/// The callback returns a list of output lines that is then compared to the
/// lines loaded from a `.valid` file with the same name as the input.
///
/// The test will fail if the callback output does not match the lines in the
/// `.valid` file. In this case, the function will output the differences
/// (see below).
///
/// After running the callback for all inputs, if there was any failed test
/// the function will panic.
///
/// ## Input lines
///
/// For convenience, both the `.input` and `.valid` files are read into lines
/// by using the [text::lines()](fn@super::text::lines) function, which
/// provides some whitespace filtering and normalization.
///
/// The use of lines is more convenient for most test cases and the filtering
/// avoids errors by differences in whitespace.
///
/// ## Failure output
///
/// After testing all `.input` files, the function will output a summary of
/// the tests. For failed tests, [diff::lines](fn@super::diff::lines) will
/// be used to provide the difference between the actual lines (`source`) and
/// the expected lines from the `.valid` file (`result`).
///
/// ## Generating valid files
///
/// As a convenience feature, if a `.valid` file is not found alongside the
/// input, the test will fail but will also create a `.valid.new` file with
/// the actual output.
///
/// This feature can be used to easily generate `.valid` files by creating
/// the `.input` file, running the tests, and then removing the `.new` from
/// the created file after manually inspecting it to make sure it is the
/// expected behavior.
pub fn testdata<P, F>(path: P, mut callback: F)
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let tests = TestData::new(path.as_ref().to_str().unwrap(), |input| {
		let output = callback(super::text::lines(input.text()));
		output.join("\n")
	});
	let result = tests.run();
	result.check();
}

pub struct TestData<T>
where
	T: FnMut(&TestInput) -> String,
{
	callback: T,
	tests: Vec<TestInput>,
}

impl<T: FnMut(&TestInput) -> String> TestData<T> {
	pub fn new<P: AsRef<str>>(source: P, callback: T) -> Self {
		let tests = collect_test_inputs(source);
		TestData { callback, tests }
	}

	pub fn run(self) -> TestRun {
		let mut output = TestRun {
			results: Vec::new(),
		};
		let mut callback = self.callback;
		for input in self.tests {
			let output_text = callback(&input);
			let output_lines = super::text::lines(&output_text);

			let mut test_succeeded = true;

			let mut valid_file_path = input.path.clone();
			valid_file_path.set_extension(TEST_VALID_FILE_EXTENSION);

			let expected_lines = match std::fs::read_to_string(&valid_file_path) {
				Ok(raw_text) => {
					let expected_lines = super::text::lines(raw_text);
					let expected_text = expected_lines.join("\n");
					let actual_text = output_lines.join("\n");
					if actual_text != expected_text {
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
						panic!("failed to read output file for {}: {}", &input.name, err);
					}

					// there is no expected lines in this case, since the valid
					// file was not found
					None
				}
			};

			let valid_file_name = valid_file_path.file_name().unwrap().to_string_lossy();
			output.results.push(TestResult {
				success: test_succeeded,
				name: input.name,
				valid_file: valid_file_name.into(),
				expect: expected_lines,
				actual: output_lines,
			});
		}
		output
	}
}

pub struct TestInput {
	name: String,
	path: PathBuf,
	text: String,
}

impl TestInput {
	pub fn name(&self) -> &str {
		&self.name
	}

	pub fn text(&self) -> String {
		self.text.clone()
	}
}

pub struct TestRun {
	results: Vec<TestResult>,
}

impl TestRun {
	pub fn check(&self) {
		let mut failed_count = 0;

		for it in &self.results {
			if it.success {
				println!("passed: {}", it.name);
			} else {
				println!("failed: {}", it.name);
				failed_count += 1;
			}
		}

		if failed_count > 0 {
			for it in &self.results {
				if !it.success {
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
			for it in &self.results {
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

	pub fn success(&self) -> bool {
		self.results.iter().all(|x| x.success)
	}

	pub fn all(&self) -> Vec<&TestResult> {
		self.results.iter().collect()
	}

	pub fn failed(&self) -> Vec<&TestResult> {
		self.results.iter().filter(|x| !x.success).collect()
	}
}

#[derive(Debug, Clone)]
pub struct TestResult {
	success: bool,

	/// The test case name. This is the input file name, without the base path.
	name: String,

	/// Name for the valid file containing the expected test output.
	valid_file: String,

	/// Expected test output from the valid file. This will be `None` if the
	/// test failed because the valid file was not found.
	expect: Option<Vec<String>>,

	/// Actual output form the test callback.
	actual: Vec<String>,
}

impl TestResult {
	pub fn name(&self) -> &str {
		&self.name
	}
}

fn collect_test_inputs<P: AsRef<str>>(source: P) -> Vec<TestInput> {
	let root_path = PathBuf::from(source.as_ref());
	let mut test_inputs = Vec::new();

	struct Directory {
		name: String,
		path: PathBuf,
	}

	let mut directories_to_scan = VecDeque::new();
	directories_to_scan.push_back(Directory {
		path: root_path,
		name: String::new(),
	});

	while let Some(next_directory) = directories_to_scan.pop_front() {
		let current_dir = next_directory.path;
		let current_name = next_directory.name;
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
				directories_to_scan.push_back(Directory {
					name: entry_name,
					path: entry_path,
				});
			} else if let Some(extension) = entry_path.extension() {
				if extension == TEST_INPUT_FILE_EXTENSION {
					test_inputs.push(TestInput {
						name: entry_name,
						text: std::fs::read_to_string(&entry_path).expect("reading test input"),
						path: entry_path,
					});
				}
			}
		}
	}

	test_inputs
}

pub fn testdata_to_result<P, F>(path: P, mut callback: F) -> TestRun
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let tests = TestData::new(path.as_ref().to_str().unwrap(), |input| {
		let output = callback(super::text::lines(input.text()));
		output.join("\n")
	});
	let result = tests.run();
	result
}

#[cfg(test)]
#[cfg(feature = "temp")] // we use `temp` in the tests
mod test_testdata {
	use super::testdata;
	use crate::{temp_dir, testdata_to_result, TempDir};

	#[test]
	fn runs_test_callback() {
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
	fn runs_test_callback_with_input() {
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
	fn fails_if_output_is_missing() {
		let dir = temp_dir();
		dir.create_file("test.input", "some input");

		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	#[test]
	fn fails_if_output_is_different() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "some input", "some output");

		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	#[test]
	fn runs_test_callback_for_each_input() {
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
	fn recurses_into_subdirectories() {
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
	fn fails_and_generate_an_output_file_if_one_does_not_exist() {
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

	#[test]
	fn trims_input_files() {
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
	fn trims_expected_output_files() {
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
	fn ignores_line_break_differences_in_input_and_output() {
		let dir = temp_dir();
		helper::write_case(&dir, "a.input", "a\nb\nc", "c\r\nb\r\na");
		helper::write_case(&dir, "b.input", "a\r\nb\r\nc", "c\nb\na");

		testdata(dir.path(), |mut input| {
			input.reverse();
			input
		});
	}

	#[test]
	fn does_not_ignore_trailing_indentation_of_first_line() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "value", "  value");
		let res = testdata_to_result(dir.path(), |input| input);
		assert!(!res.success());
	}

	//------------------------------------------------------------------------//
	// TestDataResult
	//------------------------------------------------------------------------//

	#[test]
	fn to_result_returns_ok_for_valid_case() {
		let dir = temp_dir();
		helper::write_case(&dir, "test.input", "abc\n123", "123\nabc");

		let result = testdata_to_result(dir.path(), |mut input| {
			input.reverse();
			input
		});

		assert!(result.success());
		assert_eq!(result.all().len(), 1);
		assert_eq!(result.all()[0].name, "test.input");
		assert_eq!(result.all()[0].success, true);
	}

	#[test]
	fn to_result_returns_an_item_for_each_case() {
		let dir = temp_dir();
		helper::write_case(&dir, "a.input", "A", "a");
		helper::write_case(&dir, "b.input", "B", "b");
		helper::write_case(&dir, "sub/some.input", "Some", "some");

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});

		assert_eq!(result.all().len(), 3);
		assert_eq!(result.all()[0].name, "a.input");
		assert_eq!(result.all()[1].name, "b.input");
		assert_eq!(result.all()[2].name, "sub/some.input");
	}

	#[test]
	fn to_result_fails_if_output_does_not_match() {
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
		assert!(result.all().len() == 3);
		assert!(result.all()[0].success);
		assert!(result.all()[1].success);
		assert!(!result.all()[2].success);
	}

	//------------------------------------------------------------------------//
	// Helper code
	//------------------------------------------------------------------------//

	mod helper {
		use super::*;

		pub fn write_case(dir: &TempDir, input_file: &str, input: &str, expected: &str) {
			dir.create_file(input_file, input);

			let suffix = format!(".input");
			let basename = input_file.strip_suffix(&suffix).unwrap();
			dir.create_file(&format!("{}.valid", basename), expected);
		}
	}
}
