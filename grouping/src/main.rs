use std::{
	collections::HashMap,
	fs::File,
	io::{BufRead, BufReader, BufWriter, Write},
	process::exit,
};

use anyhow::anyhow;
use clap::Parser;
use indexmap::IndexMap;
use operations::{Operation, OperationFunction};

mod operations;

#[derive(Parser)]
pub struct Arguments {
	/// File to be grouped
	#[arg(short, long)]
	in_file: String,

	/// File to write to
	#[arg(short, long)]
	out_file: String,

	/// Column to group by
	#[arg(short, long)]
	group_by: usize,

	/// Whether to ignore case when grouping
	#[arg(short, long)]
	ignore_case: bool,

	/// Comma separated list of ascii characters. Rows starting with any of the characters will be ignored
	#[arg(short, long, num_args = 0.., value_delimiter = ',')]
	delete_rows: Vec<char>,

	/// Operations to run separated by a space. Format: operation,column,round_result,(optional)default_value
	#[arg(visible_alias = "ops", visible_alias = "op", long, value_parser = parse_operation, num_args = 0.., value_delimiter = ' ')]
	operations: Vec<OperationFunction>,
}

trait ToOptionF64 {
	fn to_option_f64(&self) -> Option<f64>;
}

impl ToOptionF64 for Option<&&str> {
	fn to_option_f64(&self) -> Option<f64> {
		match self {
			Some(s) => s.parse::<f64>().ok(),
			None => None,
		}
	}
}

fn parse_operation(arg: &str) -> Result<OperationFunction, anyhow::Error> {
	let parts: Vec<&str> = arg.split(',').collect();

	let min_args_err = || {
		anyhow!("Expected at least 3 comma separated arguments as an operation! Arguments: {arg}")
	};

	let op = parts.get(0).ok_or(min_args_err())?;
	let col = parts.get(1).ok_or(min_args_err())?;
	let round = parts.get(2).ok_or(min_args_err())?;
	let default = parts.get(3).to_option_f64();

	let op = match op.to_owned() {
		"mean" => Operation::Mean,
		"median" => Operation::Median,
		"mode" => Operation::Mode,
		"max" => Operation::Maximum,
		"min" => Operation::Minimum,
		"sum" => Operation::Sum,
		"length" => Operation::Count,
		"unique" => Operation::CountDistinct,
		"cat" => Operation::Concatenate,
		"cat_uniq" => Operation::ConcatenateDistinct,
		"random" => Operation::Random,
		"std" => Operation::StandardDeviation,
		s => return Err(anyhow!("{s} is not a valid operation. Valid operations are: mean, median, mode, max, min, sum, length, unique, cat, cat_uniq, random, std"))
	};

	let col = col
		.parse::<usize>()
		.or(Err(anyhow!("{col} can not be parsed as column number")))?;
	let round = round.parse::<bool>().or(Err(anyhow!(
		"{round} can not be parsed as boolean for 'round'"
	)))?;

	Ok(OperationFunction {
		op,
		col,
		round,
		default,
	})
}

fn main() {
	let args = Arguments::parse();

	match run_with_args(&args) {
		Ok(report) => println!("{report}"),
		Err(e) => {
			eprintln!("{e}");
			exit(1);
		}
	};
}

pub fn run_with_args(args: &Arguments) -> Result<String, anyhow::Error> {
	let input_file = File::open(&args.in_file)?;
	let output_file = File::create(&args.out_file)?;

	let reader = BufReader::new(input_file);
	let mut writer = BufWriter::new(output_file);

	let columns_used: Vec<usize> = args.operations.iter().map(|op_fn| op_fn.col).collect();
	let mut groups: IndexMap<String, Group> = IndexMap::new();

	for (line_number, line) in reader.lines().enumerate() {
		let line = line?;
		let values: Vec<&str> = line.split("\t").collect();

		let group_val = values
			.get(args.group_by)
			.ok_or(anyhow!(
				"Grouping column {} not defined on line {line_number}",
				args.group_by
			))?
			.to_owned();

		let group_val = if args.ignore_case {
			group_val.to_lowercase()
		} else {
			group_val.to_owned()
		};

		let group = groups.entry(group_val).or_insert(Group {
			columns: HashMap::new(),
		});

		for col in &columns_used {
			let column = group.columns.entry(*col).or_insert(Vec::new());
			let val = match values.get(*col) {
				Some(s) => s,
				None => "",
			};

			column.push(val.to_owned());
		}
	}

	for (key, group) in &groups {
		let mut outputs = Vec::new();
		outputs.push(key.to_owned());

		for op_fn in &args.operations {
			let values = group
				.columns
				.get(&op_fn.col)
				.ok_or(anyhow!("Internal Error. HashMap improperly populated"))?;
			let output = op_fn.run_operation(values);
			outputs.push(output);
		}

		writer.write(&format!("{}\n", outputs.join("\t")).into_bytes())?;
	}

	writer.flush()?;

	Ok(format!("Grouped into {} lines", groups.len()))
}

struct Group {
	columns: HashMap<usize, Vec<String>>,
}
