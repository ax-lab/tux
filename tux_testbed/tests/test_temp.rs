use tux::*;

#[test]
fn test_temp_dir_run_bin_executes_in_temp_dir() {
	let data = TestTempDir::create_new();
	data.create_file("test.txt", "test file data");
	let output = data.run_bin("bin_simple", &["test.txt"]);
	assert!(
		output.contains("test file data"),
		"unexpected output: {}",
		output
	);
}
