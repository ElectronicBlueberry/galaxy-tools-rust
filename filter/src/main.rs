use anyhow::anyhow;
use clap::{Parser, ValueEnum};
use evalexpr::{build_operator_tree, ContextWithMutableVariables, HashMapContext, Node, Value};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::process::exit;

mod functions;
use crate::functions::create_function_context;

#[cfg(test)]
mod tests;

#[derive(Parser)]
pub struct Arguments {
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
pub enum ColumnType {
	Str,
	Int,
	Float,
	Bool,
	None,
	List,
}

fn main() {
	let args = Arguments::parse();

	match run_with_args(&args) {
		Ok(report) => print!("{report}"),
		Err(e) => {
			eprintln!("{e}");
			exit(1);
		}
	};
}

pub fn run_with_args(args: &Arguments) -> Result<String, anyhow::Error> {
	let mut reader = create_reader(&args.in_file)?;
	let mut writer = create_writer(&args.out_file)?;

	let res = filter_with_expression(
		&mut reader,
		&mut writer,
		&args.expression,
		args.skip_lines,
		&args.types,
	);

	res
}

/// create a buffered reader from `file path`
pub fn create_reader(file_path: &String) -> Result<BufReader<File>, anyhow::Error> {
	match File::open(file_path) {
		Ok(f) => Ok(BufReader::new(f)),
		Err(_) => Err(anyhow!("Failed to open input file '{file_path}'")),
	}
}

/// create a buffered writer from `file_path`
pub fn create_writer(file_path: &String) -> Result<BufWriter<File>, anyhow::Error> {
	match File::create(file_path) {
		Ok(f) => Ok(BufWriter::new(f)),
		Err(_) => Err(anyhow!("Failed to create output file '{file_path}'")),
	}
}

/// filter `input_reader` to `output_reader` using `expression`
pub fn filter_with_expression(
	input_reader: &mut BufReader<File>,
	output_writer: &mut BufWriter<File>,
	expression: &String,
	skip_lines: usize,
	column_types: &[ColumnType],
) -> Result<String, anyhow::Error> {
	let columns = get_used_columns(expression);

	let precompiled_exp = match compile_expression(&columns, column_types, expression) {
		Ok(n) => n,
		Err(e) => return Err(e),
	};

	// Lines with invalid contents
	let mut invalid_lines: usize = 0;
	let mut first_invalid_line: usize = 0;
	let mut invalid_line_content = String::new();

	// empty or comment lines
	let mut skipped_lines: usize = 0;

	// amount of read lines. Including skipped and invalid
	let mut total_lines: usize = 0;

	let mut lines_kept: usize = 0;

	let mut ctx = create_function_context();

	for (line_number, line) in input_reader.lines().enumerate() {
		let line = match line {
			Ok(l) => l,
			Err(_) => return Err(anyhow!("Failed to read file at line number {line_number}")),
		};

		// convert given line to buffer with newline
		let buf = |line: String| format!("{line}\n").into_bytes();

		total_lines += 1;

		if line_number < skip_lines {
			match output_writer.write(&buf(line)) {
				Ok(_) => lines_kept += 1,
				Err(_) => {
					return Err(anyhow!(
						"Failed to write to output file at line number {line_number}"
					))
				}
			};
			continue;
		}

		if line.trim().is_empty() || line.starts_with('#') {
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

		match mutate_context_for_line(&line, column_types, &columns, &mut ctx) {
			Ok(_) => (),
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
						"Failed to write to output file at line number {line_number}",
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
			"No lines kept. Check filter condition '{expression}', see tool tips, syntax and examples\n"
		);
	}

	if invalid_lines > 0 {
		report += &format!(
			"Skipped {invalid_lines} invalid line(s) starting at line {first_invalid_line}: '{invalid_line_content}'\n"
		);
	}

	if skipped_lines > 0 {
		report += &format!("Skipped {skipped_lines} comment (starting with #) or blank line(s)\n");
	}

	Ok(report)
}

/// compile and test run the expression to check for any errors
fn compile_expression(
	columns: &[usize],
	column_types: &[ColumnType],
	expression: &String,
) -> Result<Node, anyhow::Error> {
	let precompiled_exp = match build_operator_tree(expression) {
		Ok(n) => n,
		Err(e) => {
			return Err(anyhow!(
				"Could not compile expression '{expression}'. Please check the syntax. \n Detailed Error: \n {e}"
			))
		}
	};

	let mut mock_values = Vec::new();

	for t in column_types {
		match t {
			ColumnType::Bool => mock_values.push("true"),
			ColumnType::Float => mock_values.push("0.1"),
			ColumnType::Int => mock_values.push("0"),
			ColumnType::Str => mock_values.push("string"),
			ColumnType::None => mock_values.push(""),
			ColumnType::List => mock_values.push("1,2,3"),
		}
	}

	let mock_line = mock_values.join("\t");
	let mut context = create_function_context();
	mutate_context_for_line(&mock_line, column_types, columns, &mut context).unwrap();

	match precompiled_exp.eval_boolean_with_context(&context) {
		Ok(_) => Ok(precompiled_exp),
		Err(e) => {
			Err(anyhow!(
				"Expression test failed for expression: '{expression}'. Please check the syntax and column types. \n Detailed Error: \n {e}"
			))
		}
	}
}

/// find columns potentially used in expression
fn get_used_columns(expression: &str) -> Vec<usize> {
	let mut columns = Vec::new();
	let r = Regex::new(r"c(?P<column>[0-9]+)").unwrap();

	for captures in r.captures_iter(expression) {
		let m = captures.name("column").unwrap();
		let i = m.as_str().parse::<usize>().unwrap();
		// column syntax is 1 based, so subtract 1
		columns.push(i - 1);
	}

	columns
}

/// Mutates `context` for a line from the file, containing needed variables.
/// Returns unspecific error for invalid lines
fn mutate_context_for_line(
	line: &str,
	column_types: &[ColumnType],
	columns: &[usize],
	context: &mut HashMapContext,
) -> Result<(), anyhow::Error> {
	let split_line = line.split('\t').collect::<Vec<&str>>();

	for column in columns {
		let t = match column_types.get(*column) {
			Some(t) => t,
			None => &ColumnType::None,
		};

		// column syntax is 1 based, so add 1
		let mut set = |v: Value| context.set_value(format!("c{}", column + 1), v);

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
			ColumnType::List => set(Value::Tuple(
				str_value
					.split(',')
					.into_iter()
					.map(|s| Value::String(s.to_owned()))
					.collect(),
			))?,
		};
	}

	Ok(())
}
