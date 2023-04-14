use evalexpr::{context_map, EvalexprError, HashMapContext, Value, ValueType};

pub fn create_function_context() -> HashMapContext {
	return context_map! {
		"in" => Function::new(|argument| {
			let arguments = argument.as_fixed_len_tuple(2)?;

			match &arguments[1] {
				Value::Tuple(tuple) => Ok(Value::Boolean(tuple.contains(&arguments[0]))),
				Value::String(string) => Ok(Value::Boolean(string.contains(&arguments[0].as_string()?))),
				v => Err(EvalexprError::TypeError { expected: vec![ValueType::Tuple, ValueType::String], actual: v.to_owned() })
			}
		})
	}
	.unwrap();
}
