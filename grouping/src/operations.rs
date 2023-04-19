use gpoint::GPoint;
use indexmap::IndexSet;
use rand::seq::SliceRandom;
use std::collections::{HashMap, HashSet};

trait ToF64Vec {
	fn to_f64_vec(&self, default: Option<f64>) -> Vec<f64>;
}

impl ToF64Vec for Vec<String> {
	fn to_f64_vec(&self, default: Option<f64>) -> Vec<f64> {
		let default = default.unwrap_or(f64::NAN);

		self.iter()
			.map(|s| s.parse::<f64>().unwrap_or(default))
			.filter(|f| !f.is_nan())
			.collect()
	}
}

trait ToStringRound {
	fn to_string_round(&self, round: bool) -> String;
}

impl ToStringRound for f64 {
	fn to_string_round(&self, round: bool) -> String {
		if round {
			GPoint(self.round()).to_string()
		} else {
			GPoint(*self).to_string()
		}
	}
}

impl ToStringRound for usize {
	fn to_string_round(&self, _round: bool) -> String {
		GPoint(*self as f64).to_string()
	}
}

#[derive(Clone)]
pub enum Operation {
	Mean,
	Median,
	Mode,
	Maximum,
	Minimum,
	Sum,
	Count,
	CountDistinct,
	Concatenate,
	ConcatenateDistinct,
	Random,
	StandardDeviation,
}

impl Operation {
	fn run(&self, values: &Vec<String>, round: bool, default: Option<f64>) -> String {
		match self {
			Operation::Mean => {
				let total: f64 = values.to_f64_vec(default).into_iter().sum();

				(total / (values.len() as f64)).to_string_round(round)
			}
			Operation::Median => {
				let mut values = values.to_f64_vec(default);
				values.sort_unstable_by(|a, b| {
					a.partial_cmp(b)
						.expect("Failed to compute Median. Unexpected NaN value")
				});
				let mid = values.len() / 2;

				match values.get(mid) {
					Some(f) => f.to_string_round(round),
					None => "".to_owned(),
				}
			}
			Operation::Mode => {
				let mut counts = HashMap::new();

				for val in values.to_f64_vec(default) {
					*counts.entry(val.to_string()).or_insert(0) += 1;
				}

				let (k, _v) = counts
					.into_iter()
					.max_by_key(|(_k, v)| *v)
					.expect("Failed to compute Mode. Received no values");

				match k.parse::<f64>().ok() {
					Some(f) => f.to_string_round(round),
					None => "".to_owned(),
				}
			}
			Operation::Maximum => {
				let mut values = values.to_f64_vec(default);

				values.sort_unstable_by(|a, b| {
					a.partial_cmp(b)
						.expect("Failed to compute Maximum. Unexpected NaN value")
				});

				match values.first() {
					Some(f) => f.to_string_round(round),
					None => "".to_owned(),
				}
			}
			Operation::Minimum => {
				let mut values = values.to_f64_vec(default);

				values.sort_unstable_by(|a, b| {
					a.partial_cmp(b)
						.expect("Failed to compute Maximum. Unexpected NaN value")
				});

				match values.last() {
					Some(f) => f.to_string_round(round),
					None => "".to_owned(),
				}
			}
			Operation::Sum => values.to_f64_vec(default).into_iter().sum::<f64>().to_string_round(round),
			Operation::Count => values.len().to_string_round(round),
			Operation::CountDistinct => {
				let mut set = HashSet::new();

				for val in values {
					set.insert(val);
				}

				set.len().to_string_round(round)
			}
			Operation::Concatenate => values.join(","),
			Operation::ConcatenateDistinct => {
				let mut set = IndexSet::new();

				for val in values {
					set.insert(val.to_owned());
				}

				set.into_iter().collect::<Vec<_>>().join(",")
			}
			Operation::Random => values
				.choose(&mut rand::thread_rng())
				.expect("Failed to choose random value. Received no values")
				.to_owned(),
			Operation::StandardDeviation => {
				let values = values.to_f64_vec(default);
				let count = values.len() as f64;
				let sum: f64 = values.iter().sum();
				let sum2: f64 = values.into_iter().map(|f| f * f).sum();
				let mean = sum / count;

				f64::sqrt((sum2 / count) - (mean * mean)).to_string_round(round)
			}
		}
	}
}

#[derive(Clone)]
pub struct OperationFunction {
	pub op: Operation,
	pub col: usize,
	pub round: bool,
	pub default: Option<f64>,
}

impl OperationFunction {
	pub fn run_operation(&self, values: &Vec<String>) -> String {
		self.op.run(&values, self.round, self.default)
	}
}
