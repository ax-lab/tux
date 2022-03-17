use tux::*;

mod get_bin {
	use super::get_bin;

	#[test]
	#[should_panic = "could not find executable"]
	fn panics_if_executable_does_not_exist() {
		get_bin("does_not_exist");
	}
}

mod run_bin {
	use super::run_bin;

	#[test]
	fn executes_binary_and_returns_stdout() {
		let output = run_bin("bin_simple", &[]);
		assert!(
			output.contains("tux simple output"),
			"unexpected output: {}",
			output
		);
	}

	#[test]
	#[should_panic = "some error output"]
	fn panics_if_there_is_error_output() {
		run_bin("bin_with_error", &[]);
	}

	#[test]
	#[should_panic = "exit code: 123"]
	fn panics_on_non_zero_exit_code() {
		run_bin("bin_with_error", &["exitcode"]);
	}
}

mod get_command_output {
	use super::get_bin;
	use super::get_command_output;

	#[test]
	#[should_panic = "some error output"]
	fn panics_if_there_is_error_output() {
		let mut cmd = get_bin("bin_with_error");
		get_command_output(&mut cmd);
	}

	#[test]
	#[should_panic = "exit code: 123"]
	fn panics_on_non_zero_exit_code() {
		let mut cmd = get_bin("bin_with_error");
		cmd.args(&["exitcode"]);
		get_command_output(&mut cmd);
	}
}
