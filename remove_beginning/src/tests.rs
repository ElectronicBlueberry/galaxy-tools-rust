use std::fs::{create_dir_all, remove_dir_all};

use super::*;
use test_utils::*;

#[test]
fn remove_beginning() {
	let tmp = ".tmp/1";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/1.bed".to_owned(),
		out_file: format!("{tmp}/out.bed"),
		num_lines: 5,
	};

	let res = run_with_args(&args);

	assert!(res.is_ok());
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/remove_beginning_test1.bed"
	));

	remove_dir_all(tmp).unwrap();
}
