use tux::*;

#[test]
fn testdata_reverse_case_works() {
	testdata("tests/testdata/reverse", |mut input| {
		input.reverse();
		input
	});
}

#[test]
#[should_panic = "tests failed"]
fn testdata_failed_case_fails() {
	testdata("tests/testdata/failed", |input| input);
}

#[test]
fn testdata_can_execute_test_executable() {
	let output = run_bin("bin_testdata", &["info"]);
	assert!(output.contains("tux testdata helper"));
}
