use tux::*;

#[test]
fn test_temp_dir_run_and_get_output_runs_executable_in_temp_dir() {
	let data = TestTempDir::create_new();
	data.create_file("test.txt", "test file data");
	let output = data.run_and_get_output("bin_simple", &["test.txt"]);
	assert!(
		output.contains("test file data"),
		"unexpected output: {}",
		output
	);
}
