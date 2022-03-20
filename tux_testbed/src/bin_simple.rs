//! Basic program used to test that the library can run binaries and properly
//! retrieve the output.

fn main() {
	// this line is used by the integration tests, do not change it
	println!("tux simple output\n");

	println!("Used as part of the testing harness. Output files passed as arguments.");

	for it in std::env::args().skip(1) {
		let file = std::fs::read_to_string(it).unwrap();
		println!("{}", file);
	}
}
