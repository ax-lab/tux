use tux::*;

mod test_temp_dir {
	use super::TestTempDir;

	#[test]
	fn run_bin_executes_in_the_temporary_directory() {
		let data = TestTempDir::create_new();
		data.create_file("test.txt", "test file data");
		let output = data.run_bin("bin_simple", &["test.txt"]);
		assert!(
			output.contains("test file data"),
			"unexpected output: {}",
			output
		);
	}
}
