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

#[cfg(test)]
mod tests {
	use super::*;
	use evalexpr::eval_boolean_with_context;

	#[test]
	fn in_function_on_tuple() {
		let ctx = create_function_context();

		assert!(eval_boolean_with_context(r#"in("a", ("a", "b", "c"))"#, &ctx).unwrap());
		assert!(eval_boolean_with_context(r#"in(2, (1, 2, 3))"#, &ctx).unwrap());
		assert!(eval_boolean_with_context(r#"in("foo", (1, 2, "foo", "bar"))"#, &ctx).unwrap());

		assert_eq!(eval_boolean_with_context(r#"in("d", ("a", "b", "c"))"#, &ctx).unwrap(), false);
		assert_eq!(eval_boolean_with_context(r#"in("2", (1, 2, 3))"#, &ctx).unwrap(), false);
		assert_eq!(eval_boolean_with_context(r#"in(1, ("1", "2", "foo", "bar"))"#, &ctx).unwrap(), false);
	}

	#[test]
	fn in_function_on_string() {
		let ctx = create_function_context();

		assert!(eval_boolean_with_context(r#"in("a", "abc")"#, &ctx).unwrap());
		assert!(eval_boolean_with_context(r#"in("substring", "contains substring")"#, &ctx).unwrap());
		assert!(eval_boolean_with_context(r#"in("bar", "foo bar baz")"#, &ctx).unwrap());

		assert_eq!(eval_boolean_with_context(r#"in("d", "abc")"#, &ctx).unwrap(), false);
		assert_eq!(eval_boolean_with_context(r#"in("2", "abc")"#, &ctx).unwrap(), false);
		assert_eq!(eval_boolean_with_context(r#"in("contains substring", "substring")"#, &ctx).unwrap(), false);

		assert!(eval_boolean_with_context(r#"in(1, "abc")"#, &ctx).is_err());
	}
}
