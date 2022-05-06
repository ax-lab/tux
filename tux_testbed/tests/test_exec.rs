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
	use tux::assert_panic;

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
	fn panics_on_non_zero_exit_code() {
		let run = || run_bin("bin_with_error", &["exitcode"]);
		assert_panic!("exited with error" in run());
		assert_panic!("123" in run());
	}
}

mod get_process_output {
	use tux::assert_panic;

	use super::get_bin;
	use super::get_process_output;

	#[test]
	#[should_panic = "some error output"]
	fn panics_if_there_is_error_output() {
		let mut cmd = get_bin("bin_with_error");
		get_process_output(cmd.output().unwrap());
	}

	#[test]
	fn panics_on_non_zero_exit_code() {
		let run = || {
			let mut cmd = get_bin("bin_with_error");
			cmd.args(&["exitcode"]);
			get_process_output(cmd.output().unwrap());
		};
		assert_panic!("123" in run());
		assert_panic!("exited with error" in run());
	}
}
