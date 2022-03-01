fn main() {
	let args = std::env::args().skip(1).collect::<Vec<_>>();

	if args.len() == 1 && args[0] == "info" {
		println!("tux testdata helper");
		return;
	}

	if args.len() != 2 {
		eprintln!("invalid arguments\n");
		print_usage();
		return;
	}

	let callback = match args[0].as_str() {
		"empty" => callback_empty,
		"reverse" => callback_reverse,
		"id" => callback_id,
		_ => {
			eprintln!("invalid function: {}\n", args[0]);
			print_usage();
			std::process::exit(1);
		}
	};

	tux::testdata(&args[1], callback);

	fn callback_empty(_: Vec<String>) -> Vec<String> {
		Vec::new()
	}

	fn callback_reverse(mut input: Vec<String>) -> Vec<String> {
		input.reverse();
		input
	}

	fn callback_id(input: Vec<String>) -> Vec<String> {
		input
	}
}

fn print_usage() {
	println!("Executes the testdata function in the given directory, using the given mode.\n");
	println!("This is used as part of the test harness for tux.\n");
	println!("Usage: (empty|reverse|id) DIRECTORY");
}
