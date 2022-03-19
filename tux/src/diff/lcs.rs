/// Returns the longest common subsequence between the two input slices.
///
/// The sequence is returned as a vector of tuples containing the indexes
/// of common items between both inputs.
pub fn lcs<T>(list_a: &[T], list_b: &[T]) -> Vec<(usize, usize)>
where
	T: std::cmp::PartialEq,
{
	if list_a.len() == 0 || list_b.len() == 0 {
		return Vec::new();
	}

	// First we compute the LCS length between each suffix of A and B
	// and store them in a flattened matrix in row-major order.

	let mut max_len_for_suffixes = Vec::new();
	max_len_for_suffixes.resize(list_a.len() * list_b.len(), 0);

	let suffixes = |pos_a, pos_b| pos_a * list_b.len() + pos_b;

	let last_a = list_a.len() - 1;
	let last_b = list_b.len() - 1;

	for a in (0..list_a.len()).rev() {
		for b in (0..list_b.len()).rev() {
			let items_are_equal = list_a[a] == list_b[b];
			max_len_for_suffixes[suffixes(a, b)] = if items_are_equal {
				let not_at_the_end = a < last_a && b < last_b;
				let suffix_len = if not_at_the_end {
					max_len_for_suffixes[suffixes(a + 1, b + 1)]
				} else {
					0
				};
				1 + suffix_len
			} else {
				let len_skipping_a = if a < last_a {
					max_len_for_suffixes[suffixes(a + 1, b)]
				} else {
					0
				};
				let len_skipping_b = if b < last_b {
					max_len_for_suffixes[suffixes(a, b + 1)]
				} else {
					0
				};
				std::cmp::max(len_skipping_a, len_skipping_b)
			};
		}
	}

	// We computed the LCS length for all combinations of suffixes between A
	// and B. Now we use that to build the actual LCS by using the computed
	// lengths to decide which elements to take when we have a choice.

	let mut longest_sequence = Vec::new();

	// Iterate both lists until we reach the end of either list. When items
	// between the lists differ, we use the computed LCS length matrix to
	// decide whether the best solution skips an item from A or B.
	let mut a = 0;
	let mut b = 0;
	while a < list_a.len() && b < list_b.len() {
		if list_a[a] == list_b[b] {
			longest_sequence.push((a, b));
			a += 1;
			b += 1;
		} else {
			let not_at_the_end = a < last_a && b < last_b;
			if not_at_the_end {
				let len_skipping_a = max_len_for_suffixes[suffixes(a + 1, b)];
				let len_skipping_b = max_len_for_suffixes[suffixes(a, b + 1)];
				if len_skipping_a > len_skipping_b {
					a += 1;
				} else {
					b += 1;
				}
			} else {
				// when we reach the last item of either list, just iterate the
				// other until we have a match or exhausted both lists
				let remaining_items_are_from_a = a < last_a;
				if remaining_items_are_from_a {
					a += 1;
				} else {
					b += 1;
				}
			}
		}
	}

	longest_sequence
}

#[cfg(test)]
mod test_lcs {
	use super::*;

	#[test]
	fn of_empty() {
		let a = &[];
		let b = &[];
		let out = lcs::<u32>(a, b);
		assert!(out.len() == 0);
	}

	#[test]
	fn of_equal() {
		let a = &[1];
		let b = &[1];
		let out = lcs(a, b);
		assert_eq!(out, [(0, 0)]);

		let a = &[1, 2, 3];
		let b = &[1, 2, 3];
		let out = lcs(a, b);
		assert_eq!(out, [(0, 0), (1, 1), (2, 2)]);
	}

	#[test]
	fn with_completely_different() {
		let a = &[1];
		let b = &[2];
		let out = lcs(a, b);
		assert!(out.len() == 0);

		let a = &[1, 2, 3];
		let b = &[4, 5, 6];
		let out = lcs(a, b);
		assert!(out.len() == 0);
	}

	#[test]
	fn with_common_sequence() {
		let a = &[1, 2, 3];
		let b = &[0, 0, 1, 2, 3, 0, 0];
		let out = lcs(a, b);
		assert_eq!(out, [(0, 2), (1, 3), (2, 4)]);

		let a = &[0, 0, 1, 2, 3, 0, 0];
		let b = &[1, 2, 3];
		let out = lcs(a, b);
		assert_eq!(out, [(2, 0), (3, 1), (4, 2)]);
	}

	#[test]
	fn with_common_sub_sequence() {
		let a = &[1, 2, 3];
		let b = &[0, 0, 1, 0, 2, 0, 3, 0, 0];
		let out = lcs(a, b);
		assert_eq!(out, [(0, 2), (1, 4), (2, 6)]);

		let a = &[0, 0, 1, 0, 2, 0, 3, 0, 0];
		let b = &[1, 2, 3];
		let out = lcs(a, b);
		assert_eq!(out, [(2, 0), (4, 1), (6, 2)]);

		let a = &[0, 0, 1, 0, 2, 0, 3, 0, 0];
		let b = &[9, 9, 1, 9, 2, 9, 3, 9, 9];
		let out = lcs(a, b);
		assert_eq!(out, [(2, 2), (4, 4), (6, 6)]);
	}

	#[test]
	fn with_out_of_order() {
		let a = &[1, 2, 3, 4, 5];
		let b = &[5, 4, 2, 3, 1];
		let out = lcs(a, b);
		assert_eq!(out, [(1, 2), (2, 3)]);

		let a = &[5, 4, 2, 3, 1];
		let b = &[1, 2, 3, 4, 5];
		let out = lcs(a, b);
		assert_eq!(out, [(2, 1), (3, 2)]);
	}

	#[test]
	fn with_complex_sequence() {
		let a = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
		let b = &[2, 3, 1, 5, 6, 4, 8, 9, 7];
		let out = lcs(a, b);
		assert_eq!(out, [(1, 0), (2, 1), (4, 3), (5, 4), (7, 6), (8, 7)]);
	}
}
