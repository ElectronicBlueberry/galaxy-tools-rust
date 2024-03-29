use std::fs::{create_dir_all, remove_dir_all};

use super::*;
use test_utils::*;

#[test]
fn simple_string_equality() {
	let tmp = ".tmp/1";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/1.bed".to_owned(),
		out_file: format!("{tmp}/out1.bed"),
		expression: r#"c1=="chr22""#.to_owned(),
		skip_lines: 0,
		types: vec![
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test1.bed"
	));

	remove_dir_all(tmp).unwrap();
}

#[test]
fn complex_expression() {
	let tmp = ".tmp/2";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/7.bed".to_owned(),
		out_file: format!("{tmp}/out2.bed"),
		expression: r#"c1=="chr1" && c3-c2>=2000 && c6=="+""#.to_owned(),
		skip_lines: 0,
		types: vec![
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test2.bed"
	));

	remove_dir_all(tmp).unwrap();
}

#[test]
fn large_file() {
	let tmp = ".tmp/3";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/filter1_in3.sam".to_owned(),
		out_file: format!("{tmp}/out3.sam"),
		expression: r#"c3=="chr1" && c5>5"#.to_owned(),
		skip_lines: 0,
		types: vec![
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test3.sam"
	));

	remove_dir_all(tmp).unwrap();
}

#[test]
fn bad_input() {
	let tmp = ".tmp/4";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/filter1_inbad.bed".to_owned(),
		out_file: format!("{tmp}/out4.bed"),
		expression: r#"c1=="chr22""#.to_owned(),
		skip_lines: 0,
		types: vec![
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test4.bed"
	));

	remove_dir_all(tmp).unwrap();
}

#[test]
fn removes_header() {
	let tmp = "./tmp/5";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/filter1_in5.tab".to_owned(),
		out_file: format!("{tmp}/out5.tab"),
		expression: r#"c8>500"#.to_owned(),
		skip_lines: 1,
		types: vec![
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Float,
			ColumnType::Int,
			ColumnType::Float,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test5.tab"
	));

	remove_dir_all(tmp).unwrap();
}

#[test]
fn mixed_type() {
	let tmp = ".tmp/6";
	create_dir_all(tmp).unwrap();

	let args = Arguments {
		in_file: "../test_data/filter1_in6.bed".to_owned(),
		out_file: format!("{tmp}/out6.bed"),
		expression: r#"c2=="100%""#.to_owned(),
		skip_lines: 0,
		types: vec![
			ColumnType::Str,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
			ColumnType::Int,
			ColumnType::Str,
		],
	};

	let result = run_with_args(&args);
	assert!(result.is_ok(), "{:?}", result);
	assert!(is_file_equal(
		&args.out_file,
		"../test_data/filter1_test6.bed"
	));

	remove_dir_all(tmp).unwrap();
}
