use tux::*;

#[test]
fn run_bin_executes_binary_and_returns_stdout() {
	let output = run_bin("bin_simple", &[]);
	assert!(
		output.contains("tux simple output"),
		"unexpected output: {}",
		output
	);
}

#[test]
#[should_panic = "could not find executable"]
fn get_bin_panics_if_executable_does_not_exist() {
	get_bin("does_not_exist");
}

#[test]
#[should_panic = "some error output"]
fn run_bin_panics_if_there_is_error_output() {
	run_bin("bin_with_error", &[]);
}

#[test]
#[should_panic = "exit code: 123"]
fn run_bin_panics_on_non_zero_exit_code() {
	run_bin("bin_with_error", &["exitcode"]);
}

#[test]
#[should_panic = "some error output"]
fn get_command_output_panics_if_there_is_error_output() {
	let mut cmd = get_bin("bin_with_error");
	get_command_output(&mut cmd);
}

#[test]
#[should_panic = "exit code: 123"]
fn get_command_output_panics_on_non_zero_exit_code() {
	let mut cmd = get_bin("bin_with_error");
	cmd.args(&["exitcode"]);
	get_command_output(&mut cmd);
}
