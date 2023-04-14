use evalexpr::{context_map, HashMapContext, Value};

pub fn create_function_context() -> HashMapContext {
	return context_map! {
		"in" => Function::new(|argument| {
			let arguments = argument.as_fixed_len_tuple(2)?;
			let tuple = arguments[1].as_tuple()?;

			for value in tuple {
				if value == arguments[0] {
					return Ok(Value::Boolean(true));
				}
			}

			Ok(Value::Boolean(false))
		})
	}
	.unwrap();
}
