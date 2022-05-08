/// Asserts that an expression panics with the specified message.
///
/// This is provided as an alternative to the `#[should_panic]` attribute in
/// tests that allows for multiple tests in a single function.
///
/// This macro will also suppress the output from the default panic hook by
/// replacing it while the expression is run.
///
/// ```
/// # use tux::assert_panic;
/// fn panicky() {
///     panic!("some message");
/// }
///
/// assert_panic!("some message" in panicky());
/// ```
#[macro_export]
macro_rules! assert_panic {
	($message:literal in $code:expr) => {{
		let input = stringify!($code);
		let expected_message = $message;
		let prev_hook = std::panic::take_hook();
		std::panic::set_hook(Box::new(|_| {}));
		let result = std::panic::catch_unwind(move || $code);
		std::panic::set_hook(prev_hook);
		let err = result
			.err()
			.expect(&format!("in `{}`: expected a panic", input));
		let actual_message = if err.is::<&str>() {
			err.downcast::<&str>().unwrap().to_string()
		} else {
			err.downcast::<String>()
				.expect("panic result is not a string")
				.to_string()
		};
		if !actual_message.contains(expected_message) {
			panic!(
				"in `{}`: expected panic with `{}`, but it was `{}`",
				input, expected_message, actual_message
			);
		}
	}};
}

#[cfg(test)]
mod tests {
	use std::sync::Mutex;

	#[test]
	fn succeeds_if_panic() {
		assert_panic!("some panic" in panic!("some panic"));
	}

	#[test]
	fn calls_inner_expression() {
		let called = Mutex::new(false);
		let do_panic = || {
			let mut called = called.lock().unwrap();
			*called = true;
			drop(called);
			panic!("some panic");
		};
		assert_panic!("some panic" in do_panic());
		assert!(*called.lock().unwrap());
	}

	#[test]
	#[should_panic = "expected a panic"]
	fn fails_if_expression_does_not_panic() {
		assert_panic!("some error" in "does not panic");
	}

	#[test]
	#[should_panic = "`2 + 2`"]
	fn fails_with_the_input_expression() {
		assert_panic!("some error" in 2 + 2);
	}

	#[test]
	#[should_panic = "expected panic with `some panic`, but it was `other panic`"]
	fn fails_if_panic_message_does_not_match() {
		assert_panic!("some panic" in panic!("other panic"));
	}

	#[test]
	fn supports_formatted_panic_messages() {
		let value = 123;
		assert_panic!("panic with 123" in panic!("panic with {}", value));
	}

	#[test]
	fn supports_mutable_expressions() {
		let mut list = Vec::new();
		assert_panic!("bounds" in {
			list.push(123);
			let _ = list[1];
		});
	}
}
