pub fn lcs<T>(a: &[T], b: &[T]) -> Vec<(usize, usize)>
where
	T: std::cmp::PartialEq,
{
	if a.len() == 0 || b.len() == 0 {
		return Vec::new();
	}

	let mut seq_len = Vec::new();
	seq_len.resize(a.len() * b.len(), 0);

	let index = |i, j| i * b.len() + j;

	let last_a = a.len() - 1;
	let last_b = b.len() - 1;
	for i in (0..a.len()).into_iter().rev() {
		for j in (0..b.len()).into_iter().rev() {
			let pos = index(i, j);
			if a[i] == b[j] {
				seq_len[pos] = 1 + if i < last_a && j < last_b {
					seq_len[index(i + 1, j + 1)]
				} else {
					0
				};
			} else {
				seq_len[pos] = std::cmp::max(
					if i < last_a {
						seq_len[index(i + 1, j)]
					} else {
						0
					},
					if j < last_b {
						seq_len[index(i, j + 1)]
					} else {
						0
					},
				);
			}
		}
	}

	let mut out = Vec::new();
	let mut i = 0;
	let mut j = 0;
	while i < a.len() && j < b.len() {
		if a[i] == b[j] {
			out.push((i, j));
			i += 1;
			j += 1;
		} else if i < last_a && j < last_b {
			if seq_len[index(i + 1, j)] > seq_len[index(i, j + 1)] {
				i += 1;
			} else {
				j += 1;
			}
		} else if i < last_a {
			i += 1;
		} else {
			j += 1;
		}
	}
	out
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn lcs_of_empty() {
		let a = &[];
		let b = &[];
		let out = lcs::<u32>(a, b);
		assert!(out.len() == 0);
	}

	#[test]
	fn lcs_of_equal() {
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
	fn lcs_of_different() {
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
	fn lcs_with_common_sequence() {
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
	fn lcs_with_common_sub_sequence() {
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
	fn lcs_with_out_of_order() {
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
	fn lcs_with_complex_sequence() {
		let a = &[1, 2, 3, 4, 5, 6, 7, 8, 9];
		let b = &[2, 3, 1, 5, 6, 4, 8, 9, 7];
		let out = lcs(a, b);
		assert_eq!(out, [(1, 0), (2, 1), (4, 3), (5, 4), (7, 6), (8, 7)]);
	}
}
