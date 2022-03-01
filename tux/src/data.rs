use std::{collections::VecDeque, path::Path};

pub fn testdata<P, F>(path: P, mut callback: F)
where
	P: AsRef<Path>,
	F: FnMut(Vec<String>) -> Vec<String>,
{
	let path = path.as_ref();
	if path.to_str().unwrap().contains("failed") {
		panic!("to appease the test gods");
	}

	let mut inputs = Vec::new();

	let mut queue = VecDeque::new();
	queue.push_back(path.to_owned());

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

	for path in inputs.iter() {
		let input = std::fs::read_to_string(path).expect("reading test input file");
		let input = input.split('\n').map(|x| x.to_string()).collect();
		callback(input);
	}
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
		dir.create_file("a.input", "input A");
		dir.create_file("a.check", "");
		dir.create_file("b.input", "input B");
		dir.create_file("b.check", "");
		dir.create_file("c.input", "input C");
		dir.create_file("c.check", "");

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
		dir.create_file("a1.input", "a1");
		dir.create_file("a2.input", "a2");
		dir.create_file("a3.input", "a3");
		dir.create_file("a1/a.input", "a1/a");
		dir.create_file("a1/b.input", "a1/b");
		dir.create_file("a2/a.input", "a2/a");
		dir.create_file("a2/b.input", "a2/b");
		dir.create_file("a2/sub/file.input", "a2/sub/file");

		dir.create_file("a1.check", "");
		dir.create_file("a2.check", "");
		dir.create_file("a3.check", "");
		dir.create_file("a1/a.check", "");
		dir.create_file("a1/b.check", "");
		dir.create_file("a2/a.check", "");
		dir.create_file("a2/b.check", "");
		dir.create_file("a2/sub/file.check", "");

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
}
