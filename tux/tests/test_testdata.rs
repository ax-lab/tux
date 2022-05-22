use tux::*;

#[test]
fn succeeds_empty_test() {
	let tests = TestData::new("tests/testdata/empty", |input| input.text());
	let result = tests.run();
	assert!(result.success());
	assert!(result.all().len() == 0);
	result.check();
}

#[test]
fn fails_with_failing_test() {
	let tests = TestData::new("tests/testdata/failing", |input| input.text());
	let result = tests.run();
	assert!(!result.success());
	assert_eq!(result.all().len(), 2);
	assert_eq!(result.failed().len(), 1);
	assert_eq!(result.failed()[0].name(), "fail.input");
	assert_panic!("1 test case failed" in result.check());
}

#[test]
fn runs_all_tests_in_the_directory() {
	let mut test_names = Vec::new();
	let tests = TestData::new("tests/testdata/many", |input| {
		test_names.push(format!("{} ({})", input.name(), input.text().trim()));
		input.text()
	});

	let result = tests.run();
	assert!(result.success());
	assert_eq!(result.all().len(), 7);
	assert_eq!(result.failed().len(), 0);
	assert_eq!(
		test_names,
		vec![
			"01.input (01)",
			"02.input (02)",
			"a/01.input (a-01)",
			"a/02.input (a-02)",
			"b/01.input (b-01)",
			"b/02.input (b-02)",
			"c/sub/some.input (some input in c)"
		]
	)
}
