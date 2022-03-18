use tux::*;

mod testdata {
	use super::get_bin;
	use super::run_bin;
	use super::temp_dir;
	use super::testdata;

	#[test]
	fn successful_case_does_not_panic() {
		testdata("tests/testdata/reverse", |mut input| {
			input.reverse();
			input
		});
	}

	#[test]
	#[should_panic = "tests failed"]
	fn failed_case_panics() {
		testdata("tests/testdata/failed", |input| input);
	}

	#[test]
	fn can_execute_test_executable() {
		let output = run_bin("bin_testdata", &["info"]);
		assert!(output.contains("tux testdata helper"));
	}

	#[test]
	fn should_output_each_passed_test() {
		let dir = temp_dir();
		dir.create_file("a.input", "");
		dir.create_file("b.input", "");
		dir.create_file("c.input", "");
		dir.create_file("sub/some.input", "");

		dir.create_file("a.valid", "");
		dir.create_file("b.valid", "");
		dir.create_file("c.valid", "");
		dir.create_file("sub/some.valid", "");

		let output = run_bin("bin_testdata", &["empty", dir.path_str()]);

		let output = output
			.lines()
			.filter(|x| x.contains("passed") && x.contains(".input"))
			.map(|s| s.into())
			.collect::<Vec<String>>();
		assert!(output.len() == 4);
		assert!(output[0].contains("a.input"));
		assert!(output[1].contains("b.input"));
		assert!(output[2].contains("c.input"));
		assert!(output[3].contains("sub/some.input"));
	}

	#[test]
	fn should_output_failed_tests_in_output() {
		let dir = temp_dir();
		dir.create_file("pass.input", "");
		dir.create_file("fail.input", "");

		dir.create_file("pass.valid", "");
		dir.create_file("fail.valid", "this will fail");

		let output = get_bin("bin_testdata")
			.args(&["empty", dir.path_str()])
			.output()
			.unwrap();
		let output = String::from_utf8_lossy(&output.stdout)
			.lines()
			.filter(|x| x.contains("failed") && x.contains(".input"))
			.map(|x| x.into())
			.collect::<Vec<String>>();
		assert!(output.len() == 1);
		assert!(output[0].contains("fail.input"));
	}

	#[test]
	fn should_output_failed_tests_in_summary() {
		let dir = temp_dir();
		dir.create_file("pass.input", "");
		dir.create_file("fail.input", "");

		dir.create_file("pass.valid", "");
		dir.create_file("fail.valid", "this will fail");

		let output = get_bin("bin_testdata")
			.args(&["empty", dir.path_str()])
			.output()
			.unwrap();
		let stderr = String::from_utf8_lossy(&output.stderr);
		let output = stderr
			.lines()
			.filter(|x| x.contains("Failed") || (x.starts_with("-") && x.contains(".input")))
			.collect::<Vec<_>>();
		assert_eq!(
			output,
			vec!["===== Failed tests =====", "- fail.input"],
			"expected failed test summary in stderr, but it was:\n\n----\n{}\n----\n",
			stderr
		);
	}

	#[test]
	fn should_output_diff_for_failed_test() {
		let test_result = get_bin("bin_testdata")
			.args(&["id", "tests/testdata/failed_diff"])
			.output()
			.unwrap();

		let stderr = String::from_utf8_lossy(&test_result.stderr);
		let diff_lines = stderr
			.lines()
			.filter(|x| x.contains("test line") || x.contains("output did not match"))
			.collect::<Vec<_>>();
		assert_eq!(
			diff_lines,
			vec![
				"=> `test1.input` output did not match `test1.valid`:",
				"-test line A",
				"+test line 1",
				" test line 2",
				" test line 3",
				"-test line B",
				"+test line 4",
				"+test line 5",
				" test line 6",
				"+test line 7",
				" test line 8",
				" test line 9",
				"-test line C",
				"=> `test2.input` output did not match `test2.valid`:",
				"-test line A",
				"+test line B",
			],
			"expected failed test to output diff, but it was:\n\n----\n{}\n----\n",
			stderr,
		);
	}
}
