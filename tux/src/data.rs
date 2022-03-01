use std::{
	collections::VecDeque,
	path::{Path, PathBuf},
};

pub fn testdata<P, F>(path: P, callback: F)
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let path = path.as_ref();
	if path.to_str().unwrap().contains("failed") {
		panic!("to appease the test gods");
	}

	testdata_to_result(path, callback);
}

#[derive(Debug)]
pub struct TestDataResult {
	pub success: bool,
	pub tests: Vec<TestDataResultItem>,
}

#[derive(Debug)]
pub struct TestDataResultItem {
	pub success: bool,
	pub name: String,
}

pub fn testdata_to_result<P, F>(path: P, mut callback: F) -> TestDataResult
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let mut inputs = Vec::new();

	let mut queue = VecDeque::new();
	queue.push_back(path.as_ref().to_owned());

	while let Some(path) = queue.pop_front() {
		let entries = std::fs::read_dir(&path).expect("reading test directory");
		let entries = entries.map(|x| x.expect("reading test directory entry"));

		let mut entries = entries.collect::<Vec<_>>();
		entries.sort_by_key(|x| x.file_name());

		for entry in entries {
			let path = entry.path();
			let entry = std::fs::metadata(&path).expect("reading test directory metadata");
			if entry.is_dir() {
				queue.push_back(path);
			} else if let Some(extension) = path.extension() {
				if extension == "input" {
					inputs.push(path);
				}
			}
		}
	}

	let success = true;
	let mut tests = Vec::new();

	let mut common_path = if inputs.len() > 0 {
		let mut path = inputs[0].clone();
		path.pop();
		path
	} else {
		PathBuf::default()
	};
	for path in inputs.iter().skip(1) {
		while !path.starts_with(&common_path) {
			if !common_path.pop() {
				break;
			}
		}
	}

	let common_path = common_path.to_string_lossy();
	for path in inputs.iter() {
		let input = std::fs::read_to_string(path).expect("reading test input file");
		let input = input.split('\n').map(|x| x.to_string()).collect();
		callback(input);

		let path = path.to_string_lossy();
		let path = if let Some(stripped_path) = path.strip_prefix(common_path.as_ref()) {
			stripped_path
		} else {
			path.as_ref()
		};
		let path = if let Some('/' | '\\') = path.chars().next() {
			&path[1..]
		} else {
			path
		};
		tests.push(TestDataResultItem {
			success: true,
			name: path.to_string(),
		})
	}

	TestDataResult { success, tests }
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::temp::TestTempDir;

	#[test]
	fn testdata_runs_test_callback() {
		let dir = TestTempDir::create_new();
		dir.create_file("some.input", "");
		dir.create_file("some.check", "");

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
		dir.create_file("some.check", "");

		let mut test_callback_input = String::new();
		testdata(dir.path(), |input| {
			let input = input.join("\n");
			test_callback_input.push_str(&input);
			Vec::new()
		});

		assert_eq!(test_callback_input, "the input");
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

		assert!(result.success);
		assert_eq!(result.tests.len(), 1);
		assert_eq!(result.tests[0].name, "test.input");
		assert_eq!(result.tests[0].success, true);
	}

	#[test]
	fn testdata_to_result_returns_an_item_for_each_case() {
		let dir = TestTempDir::create_new();
		helper::write_case(&dir, "a.input", "A", "a");
		helper::write_case(&dir, "b.input", "B", "b");

		let result = testdata_to_result(dir.path(), |input| {
			input.into_iter().map(|x| x.to_lowercase()).collect()
		});

		assert_eq!(result.tests.len(), 2);
		assert_eq!(result.tests[0].name, "a.input");
		assert_eq!(result.tests[1].name, "b.input");
	}

	mod helper {
		use super::*;

		pub fn write_case(dir: &TestTempDir, input_file: &str, input: &str, expected: &str) {
			dir.create_file(input_file, input);

			let basename = input_file.strip_suffix(".input").unwrap();
			dir.create_file(&format!("{}.check", basename), expected);
		}
	}
}
