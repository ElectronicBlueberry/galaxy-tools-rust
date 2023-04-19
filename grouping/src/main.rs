use anyhow::anyhow;
use clap::Parser;
use operations::{Operation, OperationFunction};

mod operations;

#[derive(Parser)]
struct Arguments {
	/// File to be grouped
	#[arg(short, long)]
	in_file: String,

	/// File to write to
	#[arg(short, long)]
	out_file: String,

	/// Column to group by
	#[arg(short, long)]
	grouping_column: usize,

	/// Whether to ignore case when grouping
	#[arg(short, long)]
	ignore_case: bool,

	/// Comma separated list of ascii characters. Rows starting with any of the characters will be ignored
	#[arg(short, long, num_args = 0.., value_delimiter = ',')]
	delete_rows: Vec<char>,

	/// Operations to run separated by a space. Format: operation,column,round_result,(optional)default_value
	#[arg(short, long, value_parser = parse_operation, num_args = 1.., value_delimiter = ' ')]
	operations: Vec<OperationFunction>
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
}
