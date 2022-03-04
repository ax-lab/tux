use tux::*;

#[test]
fn get_project_bin_output_executes_binary_and_returns_stdout() {
	let output = get_project_bin_output("bin_simple", &[]);
	assert!(
		output.contains("tux simple output"),
		"unexpected output: {}",
		output
	);
}

#[test]
#[should_panic = "could not find executable"]
fn get_project_bin_panics_if_executable_does_not_exist() {
	get_project_bin("does_not_exist");
}

#[test]
#[should_panic = "some error output"]
fn get_project_bin_output_panics_if_there_is_error_output() {
	get_project_bin_output("bin_with_error", &[]);
}

#[test]
#[should_panic = "exit code: 123"]
fn get_project_bin_output_panics_on_non_zero_exit_code() {
	get_project_bin_output("bin_with_error", &["exitcode"]);
}

#[test]
#[should_panic = "some error output"]
fn get_command_output_panics_if_there_is_error_output() {
	let mut cmd = get_project_bin("bin_with_error");
	get_command_output(&mut cmd);
}

#[test]
#[should_panic = "exit code: 123"]
fn get_command_output_panics_on_non_zero_exit_code() {
	let mut cmd = get_project_bin("bin_with_error");
	cmd.args(&["exitcode"]);
	get_command_output(&mut cmd);
}
