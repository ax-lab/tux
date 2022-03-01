use tux::*;

#[test]
fn run_and_get_output_returns_stdout() {
	let output = run_and_get_output("tux_test_simple", &[]);
	assert!(
		output.contains("tux simple output"),
		"unexpected output: {}",
		output
	);
}

#[test]
fn test_data_dir_run_and_get_output_runs_executable_in_temp_dir() {
	let data = TestTempDir::create_new();
	data.create_file("test.txt", "test file data");
	let output = data.run_and_get_output("tux_test_simple", &["test.txt"]);
	assert!(
		output.contains("test file data"),
		"unexpected output: {}",
		output
	);
}
