//! This library provides miscellaneous test utility functions.

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
	let exe_path = std::env::current_exe().expect("current executable");
	let exe_path = exe_path.parent().expect("current executable path");
	let exe_path = exe_path.parent().expect("target executable path");

	let mut exe_path = exe_path.to_owned();
	exe_path.push(cmd);
	exe_path.set_extension(std::env::consts::EXE_EXTENSION);

	Command::new(exe_path)
}

/// Runs a cargo-built executable from an integration test and returns the
/// executable output as a string.
pub fn run_and_get_output(cmd: &str) -> String {
	let output = get_exe_command(cmd).output().expect("executing command");
	let stderr = String::from_utf8(output.stderr).expect("reading command stderr as UTF-8");
	assert!(stderr.len() == 0);
	String::from_utf8(output.stdout).expect("reading command stdout as UTF-8")
}
