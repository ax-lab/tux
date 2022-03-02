fn main() {
	let mut args = std::env::args().skip(1);
	if let Some(arg) = args.next() {
		if arg == "exitcode" {
			std::process::exit(1);
		}
	}

	eprintln!("some error output");
}
