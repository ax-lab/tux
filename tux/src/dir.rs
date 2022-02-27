use std::path::{Path, PathBuf};

use path_clean::PathClean;

pub struct TestDataDir {
	dir: tempfile::TempDir,
}

impl TestDataDir {
	pub fn create_new() -> TestDataDir {
		TestDataDir {
			dir: tempfile::tempdir().expect("creating temp dir for test"),
		}
	}

	pub fn path(&self) -> &Path {
		self.dir.path()
	}

	pub fn create_file<S: AsRef<[u8]>>(&self, name: &str, text: S) -> PathBuf {
		let mut path = self.path().to_owned();
		path.push(name);

		let path = path.clean();
		if !path.starts_with(self.path()) {
			panic!("cannot create test file outside root dir");
		}

		let parent = path.parent().expect("parent dir for new test file");
		std::fs::create_dir_all(parent).expect("creating parent dir for new test file");

		std::fs::write(&path, text).expect("failed to write test file");
		path
	}

	pub fn run_and_get_output(&self, cmd: &str, args: &[&str]) -> String {
		let mut cmd = super::get_exe_command(cmd);
		cmd.args(args);
		cmd.current_dir(self.path());
		super::run_command_and_get_output(&mut cmd)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_data_dir_should_create_new_directory() {
		let dir = TestDataDir::create_new();
		let path = dir.path();
		assert!(path.is_dir());
	}

	#[test]
	fn test_data_dir_should_delete_diretory_on_drop() {
		let dir = TestDataDir::create_new();
		let path = dir.path().to_owned();
		drop(dir);
		assert!(!path.exists());
	}

	#[test]
	fn test_data_dir_should_delete_diretory_on_drop_even_if_non_empty() {
		let dir = TestDataDir::create_new();
		let path = dir.path().to_owned();
		dir.create_file("root.txt", "text");
		dir.create_file("a/file.txt", "text");
		dir.create_file("b/file.txt", "text");
		dir.create_file("c/sub/file.txt", "text");
		drop(dir);
		assert!(!path.exists());
	}

	#[test]
	fn test_data_should_create_file() {
		let dir = TestDataDir::create_new();
		let file_path = dir.create_file("simple_file.txt", "123");
		assert!(file_path.is_file());

		let contents = std::fs::read_to_string(file_path).unwrap();
		assert_eq!(contents, "123");
	}

	#[test]
	fn test_data_should_create_file_with_directory() {
		let dir = TestDataDir::create_new();
		let file_path = dir.create_file("sub/simple_file.txt", "abc");
		assert!(file_path.is_file());

		let mut sub_dir = dir.path().to_owned();
		sub_dir.push("sub");
		assert!(sub_dir.is_dir());

		let contents = std::fs::read_to_string(file_path).unwrap();
		assert_eq!(contents, "abc");
	}

	#[test]
	fn test_data_should_not_create_outside_root_directory() {
		let dir = TestDataDir::create_new();
		let result = std::panic::catch_unwind(|| {
			dir.create_file(
				"sub/../../test_file.txt",
				"this test file should not be created",
			);
		});
		assert!(result.is_err());
	}
}
