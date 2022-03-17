//! This program is used to test how the library handle error conditions when
//! executing binaries.

fn main() {
	let mut args = std::env::args().skip(1);
	if let Some(arg) = args.next() {
		if arg == "exitcode" {
			std::process::exit(123);
		}
	}

	eprintln!("some error output");
}
