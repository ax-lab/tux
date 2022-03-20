use std::process::Command;

/// Returns a [`Command`] for running a binary from the project (i.e. a binary
/// built by Cargo).
///
/// This is intended to be used by integration tests that need to run one of
/// the crate's binaries.
///
/// See also [`get_process_output`] and [`run_bin`].
pub fn get_bin(name: &str) -> Command {
	// Cargo generates integration tests at `target/debug/deps`
	let mut exe_path = std::env::current_exe().expect("getting current executable filename");
	exe_path.pop();
	if exe_path.ends_with("deps") {
		exe_path.pop();
	}

	exe_path.push(name);
	exe_path.set_extension(std::env::consts::EXE_EXTENSION);

	if !exe_path.is_file() {
		panic!(
			"could not find executable for `{}` in the build directory",
			name
		);
	}

	Command::new(exe_path)
}

/// Convenience function combining [`get_bin`] and [`get_process_output`].
pub fn run_bin(cmd: &str, args: &[&str]) -> String {
	let mut cmd = get_bin(cmd);
	cmd.args(args);

	let output = cmd.output().expect("running binary");
	get_process_output(output)
}

/// Utility function to retrieve the standard output of a process from
/// the [`std::process::Output`] while validating the exit status and
/// error output.
///
/// This will panic if the process exit status is non-zero or if any
/// error output is generated.
pub fn get_process_output(output: std::process::Output) -> String {
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
