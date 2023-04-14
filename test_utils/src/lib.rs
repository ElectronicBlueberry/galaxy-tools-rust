use std::fs;

/// Checks whether the contents of two files are equal
pub fn is_file_equal(path_a: &str, path_b: &str) -> bool {
	let data_a = fs::read_to_string(path_a).expect("Unable to read first file");
	let data_b = fs::read_to_string(path_b).expect("Unable to read second file");

	data_a == data_b
}
