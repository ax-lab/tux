use std::process::Command;

/// Creates a `Command` for running the given cargo-built executable from an
/// integration test.
///
/// This is intended to be used by integration tests inside an executable
/// crate that need to run the main executable.
///
/// See `run_command_and_get_output`.
pub fn get_exe_command(cmd: &str) -> Command {
	// Cargo generates integration tests at `target/debug/deps`
	let mut exe_path = std::env::current_exe().expect("getting current executable filename");
	exe_path.pop();
	if exe_path.ends_with("deps") {
		exe_path.pop();
	}

	exe_path.push(cmd);
	exe_path.set_extension(std::env::consts::EXE_EXTENSION);

	Command::new(exe_path)
}

/// Runs a cargo-built executable from an integration test and returns the
/// executable output as a string.
///
/// This is a wrapper around `get_exe_command` and `run_command_and_get_output`.
pub fn run_and_get_output(cmd: &str, args: &[&str]) -> String {
	let mut cmd = get_exe_command(cmd);
	cmd.args(args);
	run_command_and_get_output(&mut cmd)
}

/// Runs the given `Command` and returns the output as a string. This will
/// also panic if the commands generates any error output.
pub fn run_command_and_get_output(cmd: &mut Command) -> String {
	let output = cmd.output().expect("executing executable");
	let stderr = String::from_utf8_lossy(&output.stderr);
	if !output.status.success() {
		panic!(
			"executable exited with error ({}){}",
			output.status,
			if stderr.len() > 0 {
				format!(" and error output: {}", stderr)
			} else {
				"".into()
			}
		);
	} else if stderr.len() > 0 {
		panic!("executable generated error output: {}", stderr);
	}
	String::from_utf8(output.stdout).expect("reading output as utf-8")
}
