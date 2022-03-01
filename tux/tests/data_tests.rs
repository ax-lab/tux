use tux::*;

#[test]
fn testdata_reverse_case_works() {
	testdata("tests/testdata/reverse", |mut input| {
		input.reverse();
		input
	});
}

#[test]
#[should_panic]
fn testdata_failed_case_fails() {
	testdata("tests/testdata/failed", |input| input);
}
