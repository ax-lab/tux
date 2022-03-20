use std::path::{Path, PathBuf};

// Rust's standard library option for path normalization (`canonicalize`)
// requires that the file exists.
use path_clean::PathClean;

/// Generates a temporary directory that can be used by tests. Returns
/// a [`TempDir`] value wrapping it.
///
/// The directory and its contents will be deleted once the value is
/// dropped.
///
/// # Errors
///
/// This will panic if the directory creation fails.
pub fn temp_dir() -> TempDir {
	TempDir::create_new()
}

/// Manages a temporary directory that can be used by tests. Supports creating
/// files in the directory. Once the value is dropped the entire directory and
/// its contents are deleted.
///
/// # Examples
///
/// ```
/// use tux::temp_dir;
///
/// let dir = temp_dir();
/// dir.create_file("test.txt", "some content");
/// println!("{:?}", dir.path());
/// println!("{}", dir.path_str());
///
/// // this will delete the directory and its contents
/// drop(dir);
/// ```
pub struct TempDir {
	dir: tempfile::TempDir,
	dir_str: String,
}

impl TempDir {
	/// Creates a new instance. For convenience, use the alias [`temp_dir`].
	pub fn create_new() -> TempDir {
		let dir = tempfile::tempdir().expect("creating temp dir for test");
		let dir_str = dir.path().to_string_lossy().into();
		TempDir { dir, dir_str }
	}

	/// Absolute path to the temporary directory.
	pub fn path(&self) -> &Path {
		self.dir.path()
	}

	/// Absolute path to the temporary directory as a plain string.
	pub fn path_str(&self) -> &str {
		self.dir_str.as_str()
	}

	/// Creates a file in the temporary directory. Returns the absolute path
	/// to the created file.
	///
	/// The file `name` can contain path components for intermediate
	/// directories, and those will be created as necessary.
	///
	/// # Errors
	///
	/// - This will panic if attempting to create files outside the temporary
	///   directory.
	/// - This will panic if the file creation or writing fails.
	pub fn create_file<S: AsRef<[u8]>>(&self, name: &str, contents: S) -> PathBuf {
		let mut path = self.path().to_owned();
		path.push(name);

		// normalize the path so that we can properly check it is inside the
		// temporary directory
		let path = path.clean();
		if !path.starts_with(self.path()) {
			panic!("cannot create test file outside temp dir");
		}

		let parent = path.parent().expect("parent dir for new test file");
		std::fs::create_dir_all(parent).expect("creating parent dir for new test file");

		std::fs::write(&path, contents).expect("failed to write test file");
		path
	}

	/// Equivalent to [`run_bin`](super::run_bin) but runs the binary with the
	/// temporary directory set as current working directory.
	///
	/// To get the entire process output, including the exit code and error
	/// output, use [`get_bin_output`](Self::get_bin_output) instead.
	pub fn run_bin(&self, cmd: &str, args: &[&str]) -> String {
		let output = self.get_bin_output(cmd, args);
		super::get_process_output(output)
	}

	/// Similar to [`run_bin`](Self::run_bin) but returns the entire process
	/// output. Use this to access the exit code and error output.
	pub fn get_bin_output(&self, cmd: &str, args: &[&str]) -> std::process::Output {
		let mut cmd = super::get_bin(cmd);
		cmd.args(args);
		cmd.current_dir(self.path());
		cmd.output().expect("executing binary")
	}
}

#[cfg(test)]
mod test_temp_dir {
	use super::temp_dir;
	use super::TempDir;

	#[test]
	fn alias_returns_new_instance_with_temporary_dir_created() {
		let dir = temp_dir();
		let path = dir.path();
		assert!(path.is_dir());
	}

	#[test]
	fn create_new_creates_new_directory() {
		let dir = TempDir::create_new();
		let path = dir.path();
		assert!(path.is_dir());
	}

	#[test]
	fn deletes_diretory_on_drop() {
		let dir = TempDir::create_new();
		let path = dir.path().to_owned();
		drop(dir);
		assert!(!path.exists());
	}

	#[test]
	fn path_should_be_absolute() {
		let dir = TempDir::create_new();
		let path = dir.path();
		assert!(path.is_absolute());
	}

	#[test]
	fn creates_file_at_root() {
		let dir = TempDir::create_new();
		let file_path = dir.create_file("some_file.txt", "some file contents");
		assert!(file_path.is_file());

		let contents = std::fs::read_to_string(file_path).unwrap();
		assert_eq!(contents, "some file contents");
	}

	#[test]
	fn creates_file_in_sub_directory() {
		let dir = TempDir::create_new();
		let file_path = dir.create_file("sub/a/b/simple_file.txt", "abc");
		assert!(file_path.is_file());

		let mut sub_dir = dir.path().to_owned();
		sub_dir.push("sub");
		assert!(sub_dir.is_dir());

		let contents = std::fs::read_to_string(file_path).unwrap();
		assert_eq!(contents, "abc");
	}

	#[test]
	fn deletes_diretory_on_drop_even_if_non_empty() {
		let dir = TempDir::create_new();
		let path = dir.path().to_owned();
		dir.create_file("root.txt", "text");
		dir.create_file("a/file.txt", "text");
		dir.create_file("b/file.txt", "text");
		dir.create_file("c/sub/file.txt", "text");
		drop(dir);
		assert!(!path.exists());
	}

	#[test]
	#[should_panic = "outside temp dir"]
	fn does_not_create_file_outside_root_directory() {
		let dir = TempDir::create_new();
		dir.create_file(
			"sub/../../test_file.txt",
			"this test file should not be created",
		);
	}

	#[test]
	fn path_str_returns_the_path() {
		let dir = TempDir::create_new();
		assert!(dir.path_str() == dir.path().to_string_lossy());
	}
}
