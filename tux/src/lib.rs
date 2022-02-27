//! This library provides miscellaneous test utility functions.

mod dir;
pub use dir::*;

mod server;
pub use server::*;

use std::process::Command;

/// Creates a `Command` for running the given cargo-built executable from an
/// integration test.
///
/// This is intended to be used by integration tests inside an executable crate
/// wishing to run the main executable.
pub fn get_exe_command(cmd: &str) -> Command {
	// Cargo generates integration tests at `target/debug/deps`
	let exe_path = std::env::current_exe().expect("getting current executable filename");
	let exe_path = exe_path.parent().expect("getting current executable path");
	let exe_path = exe_path.parent().expect("getting target executable path");

	// this is now `target/debug/`
	let mut exe_path = exe_path.to_owned();
	exe_path.push(cmd);
	exe_path.set_extension(std::env::consts::EXE_EXTENSION);

	Command::new(exe_path)
}

/// Runs a cargo-built executable from an integration test and returns the
/// executable output as a string.
pub fn run_and_get_output(cmd: &str, args: &[&str]) -> String {
	let mut cmd = get_exe_command(cmd);
	cmd.args(args);
	run_command_and_get_output(&mut cmd)
}

fn run_command_and_get_output(cmd: &mut Command) -> String {
	let output = cmd.output().expect("executing executable");
	if output.stderr.len() > 0 {
		let stderr = String::from_utf8_lossy(&output.stderr);
		panic!("executable generated error output: {}", stderr);
	}
	String::from_utf8(output.stdout).expect("reading output as utf-8")
}
