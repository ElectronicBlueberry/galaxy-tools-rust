use clap::Parser;
use std::{
	fs::File,
	io::{BufRead, BufReader, BufWriter, Write},
	process::exit,
};

#[cfg(test)]
mod tests;

#[derive(Parser)]
pub struct Arguments {
	/// File to be truncated
	#[arg(short, long)]
	in_file: String,

	/// File to write to
	#[arg(short, long)]
	out_file: String,

	/// Number of lines to remove
	#[arg(short, long, default_value_t = 1)]
	num_lines: usize,
}

fn main() {
	let args = Arguments::parse();

	match run_with_args(&args) {
		Ok(_) => println!("{} lines removed", args.num_lines),
		Err(e) => {
			eprintln!("{e}");
			exit(1);
		}
	};
}

pub fn run_with_args(args: &Arguments) -> Result<(), std::io::Error> {
	let input_file = File::open(&args.in_file)?;
	let output_file = File::create(&args.out_file)?;

	let reader = BufReader::new(input_file);
	let mut writer = BufWriter::new(output_file);

	for (_, line) in reader.lines().enumerate().skip(args.num_lines) {
		let line = line?;
		writer.write(&format!("{line}\n").into_bytes())?;
	}

	writer.flush()?;

	Ok(())
}
