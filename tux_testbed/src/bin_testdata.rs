//! Wrapper around the `testdata` feature in the library. This is mainly to
//! test the failed test conditions, which are hard to test as a unit test.

fn main() {
	let args = std::env::args().skip(1).collect::<Vec<_>>();

	if args.len() == 1 && args[0] == "info" {
		// this is used by the integration tests as a self-test that the
		// executable can run
		println!("tux testdata helper");
		return;
	}

	if args.len() != 2 {
		eprintln!("invalid arguments\n");
		print_usage();
		std::process::exit(1);
	}

	let arg_callback = &args[0];
	let arg_testdata_dir = &args[1];

	let callback = match arg_callback.as_str() {
		"empty" => callback_empty,
		"reverse" => callback_reverse,
		"id" => callback_id,
		_ => {
			eprintln!("invalid function: {}\n", args[0]);
			print_usage();
			std::process::exit(1);
		}
	};

	tux::testdata(arg_testdata_dir, callback);

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
	println!("Executes the testdata tests in the given directory, using the given function.\n");
	println!("This is used as part of the test harness for tux.\n");
	println!("Usage: (empty|reverse|id) DIRECTORY");
}
