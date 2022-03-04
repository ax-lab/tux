fn main() {
	println!("tux simple output\n");
	println!("Used as part of the testing harness. Output files passed as arguments.");

	for it in std::env::args().skip(1) {
		let file = std::fs::read_to_string(it).unwrap();
		println!("{}", file);
	}
}
