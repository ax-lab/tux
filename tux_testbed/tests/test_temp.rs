use tux::*;

mod temp_dir {
	use super::TempDir;

	#[test]
	fn run_bin_executes_in_the_temporary_directory() {
		let dir = TempDir::create_new();
		dir.create_file("test.txt", "test file data");
		let output = dir.run_bin("bin_simple", &["test.txt"]);
		assert!(
			output.contains("test file data"),
			"unexpected output:\n\n-----\n{}\n-----",
			output
		);
	}

	#[test]
	fn get_bin_output_executes_in_the_temporary_directory() {
		let dir = TempDir::create_new();
		dir.create_file("test.txt", "test file data");

		let output = dir.get_bin_output("bin_simple", &["test.txt"]);
		let output = String::from_utf8_lossy(&output.stdout);
		assert!(
			output.contains("test file data"),
			"unexpected output:\n\n-----\n{}\n-----",
			output
		);
	}
}
