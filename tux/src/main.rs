fn main() {
	println!("this is tux output\n");
	println!("This is only used to test that tux can run executables from the project.");

	for it in std::env::args().skip(1) {
		let file = std::fs::read_to_string(it).unwrap();
		println!("{}", file);
	}
}
