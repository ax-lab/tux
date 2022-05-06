mod panics {
	use tux::assert_panic;

	#[test]
	fn works_outside_the_crate() {
		assert_panic!("panicked" in panic!("panicked"));
	}
}
