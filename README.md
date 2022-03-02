# tux

Simple test utilities for unit and integration tests in Rust.

This project provides miscellaneous test utilities for Rust projects. The goal
is to support a variety of test scenarios to help with test driven development.

There is no particular scope with the utilities provided, other than being 
generally useful. Utilities will be added as there is a need for them. Anything
that makes testing easier and/or more practical can be added to this project.

Here are some examples of features provided by this library:

- Running crate executables from integration tests:
  - Provides helpers for finding the executable path, running it, and 
    checking the result;
  - Useful for testing command-line utilities or hard-to-test scenarios like
    panics and console output.
- Setting up temporary data files for tests:
  - Allows creating temporary files and directories for a test;
  - Creates a new empty directory in the system's temporary directory;
  - Provides helper utilities for setting up files in the directory;
  - The temporary directory and any files inside are automatically deleted.
- File-based tests:
  - Test inputs and the respective expected outputs are stored as text files
    in a project directory;
  - Each input file is processed by a test callback that processes the input
    and generates some output;
  - The test output is then compared with the output file. If they do not
    match the test fails.
- HTTP server:
  - Provides a simple HTTP server that can be configured with custom routes;
  - The server listens to a random port that is provided to the test;
  - Allows testing HTTP and web related code.
