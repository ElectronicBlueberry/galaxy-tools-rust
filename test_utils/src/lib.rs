use std::fs;

/// Checks whether the contents of two files are equal
pub fn is_file_equal(path_a: &str, path_b: &str) -> bool {
	let data_a = fs::read_to_string(path_a).expect("Unable to read first file");
	let data_b = fs::read_to_string(path_b).expect("Unable to read second file");

	data_a == data_b
}

pub fn get_tmp_dir() -> String {
	"./.tmp".to_owned()
}

pub fn remove_tmp_dir() -> std::io::Result<()> {
	let dir = get_tmp_dir();
	fs::remove_dir_all(dir)?;
	Ok(())
}

pub fn create_tmp_dir() -> std::io::Result<()> {
	let dir = get_tmp_dir();
	fs::create_dir_all(dir)?;
	Ok(())
}
