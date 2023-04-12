use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use evalexpr::{build_operator_tree, ContextWithMutableVariables, HashMapContext, Value};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::exit;

#[derive(Parser)]
struct Arguments {
	/// File to be filtered
	#[arg(short, long)]
	in_file: String,

	/// File to write to
	#[arg(short, long)]
	out_file: String,

	/// Expression used to filter rows
	#[arg(short, long)]
	expression: String,

	/// Number of header lines to skip
	#[arg(short, long, default_value_t = 0)]
	skip_lines: usize,

	/// Comma-separated list of column types
	#[arg(short, long, value_parser = clap::value_parser!(ColumnType), num_args = 1.., value_delimiter = ',')]
	types: Vec<ColumnType>,
}

#[derive(Clone, ValueEnum)]
enum ColumnType {
	Str,
	Int,
	Float,
	Bool,
	None,
}

fn main() {
	let args = Arguments::parse();

	let mut reader = create_reader(&args.in_file);
	let mut writer = create_writer(&args.out_file);

	match filter_with_expression(
		&mut reader,
		&mut writer,
		&args.expression,
		args.skip_lines,
		&args.types,
	) {
		Ok(report) => print!("{}", report),
		Err(e) => {
			eprintln!("{}", e);
			exit(1);
		}
	};
}

/// create a buffered reader from a file path
fn create_reader(file_path: &String) -> BufReader<File> {
	let input_file = match File::open(&file_path) {
		Ok(f) => f,
		Err(_e) => {
			eprintln!("Failed to open input file '{}'", file_path);
			exit(1);
		}
	};

	return BufReader::new(input_file);
}

/// create a buffered writer from a file path
fn create_writer(file_path: &String) -> BufWriter<File> {
	let output_file = match File::create(&file_path) {
		Ok(f) => f,
		Err(_e) => {
			eprintln!("Failed to create output file '{}'", file_path);
			exit(1);
		}
	};

	return BufWriter::new(output_file);
}

/// filter input_reader to output_reader using expression
fn filter_with_expression(
	input_reader: &mut BufReader<File>,
	output_writer: &mut BufWriter<File>,
	expression: &String,
	skip_lines: usize,
	column_types: &Vec<ColumnType>,
) -> Result<String, anyhow::Error> {
	let precompiled_exp = match build_operator_tree(&expression) {
		Ok(n) => n,
		Err(e) => {
			return Err(anyhow!(
				"Could not compile expression '{}'. Please check the syntax. \n Detailed Error: \n {}",
				expression,
				e
			))
		}
	};

	let columns = get_used_columns(&expression);

	// Lines with invalid contents
	let mut invalid_lines: usize = 0;
	let mut first_invalid_line: usize = 0;
	let mut invalid_line_content = String::new();

	// empty or comment lines
	let mut skipped_lines: usize = 0;

	// amount of read lines. Including skipped and invalid
	let mut total_lines: usize = 0;

	let mut lines_kept: usize = 0;

	for (line_number, line) in input_reader.lines().enumerate() {
		let line = match line {
			Ok(l) => l,
			Err(_) => {
				return Err(anyhow!(
					"Failed to read file at line number {}",
					line_number
				))
			}
		};

		// convert given line to buffer with newline
		let buf = |line: String| {
			if lines_kept == 0 {
				line.into_bytes()
			} else {
				format!("\n{}", line).into_bytes()
			}
		};

		total_lines += 1;

		if line_number < skip_lines {
			match output_writer.write(&buf(line)) {
				Ok(_) => lines_kept += 1,
				Err(_) => {
					return Err(anyhow!(
						"Failed to write to output file at line number {}",
						line_number
					))
				}
			};
			continue;
		}

		if line.trim().is_empty() || line.starts_with("#") {
			skipped_lines += 1;
			continue;
		}

		let mut line_invalid = || {
			if invalid_lines == 0 {
				first_invalid_line = line_number;
				invalid_line_content = line.clone();
			}

			invalid_lines += 1;
		};

		let ctx = match get_context_for_line(&line, column_types, &columns) {
			Ok(c) => c,
			Err(_) => {
				line_invalid();
				continue;
			}
		};

		let passed = match precompiled_exp.eval_boolean_with_context(&ctx) {
			Ok(b) => b,
			Err(_) => {
				line_invalid();
				continue;
			}
		};

		if passed {
			match output_writer.write(&buf(line)) {
				Ok(_) => lines_kept += 1,
				Err(_) => {
					return Err(anyhow!(
						"Failed to write to output file at line number {}",
						line_number
					))
				}
			};
		}
	}

	// make sure output buffer is written to disk
	match output_writer.flush() {
		Ok(_) => (),
		Err(_e) => return Err(anyhow!("Failed to write to output file")),
	};

	let mut report = String::new();
	let valid_lines = total_lines - skipped_lines;

	if valid_lines > 0 {
		report += &format!(
			"Kept {:.2}% of {} valid lines ({} total lines)\n",
			100.0 * lines_kept as f64 / valid_lines as f64,
			valid_lines,
			total_lines
		);
	} else {
		report += &format!(
			"No lines kept. Check filter condition '{}', see tool tips, syntax and examples\n",
			expression
		);
	}

	if invalid_lines > 0 {
		report += &format!(
			"Skipped {} invalid line(s) starting at line {}: '{}'\n",
			invalid_lines, first_invalid_line, invalid_line_content
		);
	}

	if skipped_lines > 0 {
		report += &format!(
			"Skipped {} comment (starting with #) or blank line(s)\n",
			skipped_lines
		);
	}

	Ok(report)
}

/// find columns potentially used in expression
fn get_used_columns(expression: &String) -> Vec<usize> {
	let mut columns = Vec::new();
	let r = Regex::new(r"c(?P<column>[0-9]+)").unwrap();

	for captures in r.captures_iter(&expression) {
		let m = captures.name("column").unwrap();
		let i = m.as_str().parse::<usize>().unwrap();
		columns.push(i);
	}

	return columns;
}

/// Constructs a context for a line from the file, containing needed variables.
/// Returns unspecific error for invalid lines
fn get_context_for_line(
	line: &String,
	column_types: &Vec<ColumnType>,
	columns: &Vec<usize>,
) -> Result<HashMapContext, anyhow::Error> {
	let split_line = line.split("\t").collect::<Vec<&str>>();

	if split_line.len() < column_types.len() {
		return Err(anyhow!("invalid column"));
	}

	let mut context = HashMapContext::new();

	for column in columns {
		let t = match column_types.get(*column) {
			Some(t) => t,
			None => &ColumnType::None,
		};

		let mut set = |v: Value| context.set_value(format!("c{}", column), v);

		let str_value = match split_line.get(*column) {
			Some(&s) => s,
			None => {
				// if value can't be found, set it to empty
				set(Value::Empty)?;
				continue;
			}
		};

		match t {
			ColumnType::Bool => set(Value::Boolean(str_value.parse::<bool>()?))?,
			ColumnType::Float => set(Value::Float(str_value.parse::<f64>()?))?,
			ColumnType::Int => set(Value::Int(str_value.parse::<i64>()?))?,
			ColumnType::Str => set(Value::String(str_value.to_string()))?,
			ColumnType::None => set(Value::Empty)?,
		};
	}

	return Ok(context);
}
