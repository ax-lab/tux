use std::process::Command;

/// Returns a `Command` for running an executable binary from the project.
///
/// This is intended to be used by integration tests that need to run one of
/// the crate's binaries.
///
/// See also: `get_command_output`.
pub fn get_project_bin(name: &str) -> Command {
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

/// Convenience function combining `get_project_bin` and `get_command_output`.
pub fn get_project_bin_output(cmd: &str, args: &[&str]) -> String {
	let mut cmd = get_project_bin(cmd);
	cmd.args(args);
	get_command_output(&mut cmd)
}

/// Simple wrapper for running a `Command` and retrieving the output, while
/// validating the exit code and stderr output.
///
/// This will call `Command::output` and panic if the exit status is non-zero
/// or if any error output is generated.
pub fn get_command_output(cmd: &mut Command) -> String {
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
