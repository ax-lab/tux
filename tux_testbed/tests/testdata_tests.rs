use tux::*;

#[test]
fn testdata_successful_case_does_not_panic() {
	testdata("tests/testdata/reverse", |mut input| {
		input.reverse();
		input
	});
}

#[test]
#[should_panic = "tests failed"]
fn testdata_failed_case_panics() {
	testdata("tests/testdata/failed", |input| input);
}

#[test]
fn testdata_can_execute_test_executable() {
	let output = run_bin("bin_testdata", &["info"]);
	assert!(output.contains("tux testdata helper"));
}

#[test]
fn testdata_should_output_each_passed_test() {
	let dir = temp_dir();
	dir.create_file("a.input", "");
	dir.create_file("b.input", "");
	dir.create_file("c.input", "");
	dir.create_file("sub/some.input", "");

	dir.create_file("a.valid", "");
	dir.create_file("b.valid", "");
	dir.create_file("c.valid", "");
	dir.create_file("sub/some.valid", "");

	let output = run_bin("bin_testdata", &["empty", dir.path_str()]);

	let output = output
		.lines()
		.filter(|x| x.contains("passed") && x.contains(".input"))
		.map(|s| s.into())
		.collect::<Vec<String>>();
	assert!(output.len() == 4);
	assert!(output[0].contains("a.input"));
	assert!(output[1].contains("b.input"));
	assert!(output[2].contains("c.input"));
	assert!(output[3].contains("sub/some.input"));
}
