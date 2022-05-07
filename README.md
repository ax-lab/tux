# tux

[<img src="https://img.shields.io/crates/v/tux.svg?style=for-the-badge&color=success&logo=rust">](https://crates.io/crates/tux)
[<img src="https://img.shields.io/badge/DOCS.RS-tux-blue?style=for-the-badge&logo=docsdotrs">](https://docs.rs/tux)

Miscellaneous test utilities for unit and integration tests in Rust.

```toml
[dev-dependencies]
tux = { version = "0.2" }
```

The goal of this project is to support a variety of test scenarios that may
be tricky to test, such as HTTP requests, executing binaries, testing complex
output, etc.

There is no particular scope with the code utilities provided, other than being
generally useful in a unit or integration test scenario.

## Cargo Features

This crate provides features for most of its functionality. By default, all
features are enabled, except overly-specific ones that are heavy to build.

Currently, that applies only to the [`server`](#http-requests) feature, which
is the only one not enabled by default.

See the crate [docs](https://docs.rs/tux) for a description of all available
crate features.

If you want to use a few specific features and prefer to opt-in instead of
having them all enabled, you can use the following in your `Cargo.toml`:

```toml
[dev-dependencies]
tux = { version = "0.2.0", default-features = false, features = ["..."] }
```

## Examples

Here are a few examples of what the library provides. This is just a sample of
all the features, so please check the [docs](https://docs.rs/tux) to see all
that is available.

### Testing panics

The `should_panic` attribute is fine, but it is too verbose and can generate
stack traces even when the tests pass. `assert_panic` is a simple alternative
that can do the same for a single expression:

```rs
fn panicky() {
    panic!("some message: and more details");
}

assert_panic!("some message" in panicky());
assert_panic!("a panic" in panic!("this is a panic"));
```

### Running an executable from your project

Allows finding and running executables from the project. Useful for testing
command-line utilities or hard-to-test scenarios like console output.

```rs
// runs the executable, validates the exit code, and
// returns stdout (combines both functions below)
let output = run_bin("some-executable-in-your-project", &["--args"]);
println!("{}", output);

// this returns a `Command`
let mut cmd = get_bin("exe-name");
cmd.args(&["--help"]);
let output = cmd.output().unwrap();

// validates the exit code and error output, returns stdout
let output = get_process_output(output);
println!("{}", output);
```

### Creating temporary directories and files

This feature enables test scenarios that require complex file input.

```rs
// create a new temporary directory in the system tempdir
let dir = temp_dir();
println!("created at {} ({:?})", dir.path_str(), dir.path());

// create some files and directories
dir.create_file("some.txt", "file contents");
dir.create_file("sub/directory/file.txt", "created with directories");

// we can run executables from the project in the directory
dir.run_bin("my-cat", &["some.txt"]);

// delete the temporary directory with its contents on drop
drop(dir);
```

### File-based tests (testdata)

Enables file-based testing. Test cases are provided as `.input` files with the
respective expected output in `.valid` files.

This can be used in any scenario that can be represented as text.

```rs
// Scans the `reverse` directory for `.input` files and executes
// the callback with the lines for each file found.
//
// The callback output is compared to the contents of a `.valid`
// file with the same name as the input. The test passes if both
// match.
//
// If any test case fails, this will display a diff with the
// failures and panic.
testdata("tests/testdata/reverse", |mut lines| {
    // In this example the `.valid` files would contain the
    // same lines as the `.input` files, but reversed.
    lines.reverse();
    lines
});
```

This feature can be useful to test complex scenarios that would be too verbose
to test using plain assertions.

- Simplifies verbose tests by representing input and expected output as text;
- Failures output a diff, making them easy to inspect and reason about.


### HTTP requests

To use this you must enable the `server` feature.

```toml
tux = { version = "0.2.0", features = ["server"] }
```

Provides a simple HTTP server powered by [warp](https://docs.rs/warp/) to test
scenarios such as a web client library.

```rs
// create a server that always responds with "some data"
let server = TestServer::new_with_root_response("some data");

// the listen port is random to avoid conflicts
let addr = format!("http://127.0.0.1:{}/", server.port());

// this is the "library" we are testing
let output = make_get_request_here(addr);
assert_eq!(output, "some data");


// another constructor, this one will respond in the `ping`
// path with information about the request
let server = TestServer::new_with_ping_route("ping");

// the server shuts down cleanly on drop
drop(server);
```

Custom routes are supported with the `new_with_routes` constructor.
