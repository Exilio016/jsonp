pub mod json_element;
pub mod parser;
mod token;

#[cfg(test)]
mod tests {
    use crate::json_element::JsonElement;
    use crate::parser::Parser;

    macro_rules! unwrap_json_element_result {
        ($element: expr, $pat: pat, $rule: block) => {
            match $element {
                Ok(e) => match e {
                    $pat => $rule,
                    _ => assert!(false),
                },
                Err(e) => {
                    let msg = e.details;
                    println!("{msg}");
                    assert!(false);
                }
            }
        };
    }

    macro_rules! unwrap_json_element {
        ($element: expr, $pat: pat, $rule: block) => {
            match $element {
                $pat => $rule,
                _ => assert!(false),
            }
        };
    }

    #[test]
    fn should_parse_num() {
        unwrap_json_element_result!(Parser::parse("14.5E-10"), JsonElement::Number(n), {
            assert_eq!(14.5E-10, n)
        });
        unwrap_json_element_result!(Parser::parse("-15"), JsonElement::Number(n), {
            assert_eq!(-15.0, n)
        });
        unwrap_json_element_result!(Parser::parse("15e7"), JsonElement::Number(n), {
            assert_eq!(15e7, n)
        });
        unwrap_json_element_result!(Parser::parse("15.7"), JsonElement::Number(n), {
            assert_eq!(15.7, n)
        });
        unwrap_json_element_result!(Parser::parse("0"), JsonElement::Number(n), {
            assert_eq!(0.0, n)
        });
        unwrap_json_element_result!(Parser::parse("-0"), JsonElement::Number(n), {
            assert_eq!(-0.0, n)
        });
        unwrap_json_element_result!(Parser::parse("-0.156"), JsonElement::Number(n), {
            assert_eq!(-0.156, n)
        });
        match Parser::parse("04") {
            Err(_) => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn should_parse_string() {
        unwrap_json_element_result!(
            Parser::parse("\"Test\\tString\\nSecond\rLine\\n\\\\\""),
            JsonElement::Str(s),
            { assert_eq!("Test\tString\nSecond\rLine\n\\", s) }
        );
    }

    #[test]
    fn should_parse_boolean() {
        unwrap_json_element_result!(Parser::parse("true"), JsonElement::Boolean(b), {
            assert!(b)
        });
        unwrap_json_element_result!(Parser::parse("false"), JsonElement::Boolean(b), {
            assert!(!b)
        });
    }

    #[test]
    fn should_parse_null() {
        unwrap_json_element_result!(Parser::parse("null"), JsonElement::Null, { assert!(true) });
    }

    #[test]
    fn should_parse_array() {
        unwrap_json_element_result!(Parser::parse("[ ]"), JsonElement::Array(a), {
            assert_eq!(0, a.len())
        });
        unwrap_json_element_result!(
            Parser::parse("[true, {\"name\":\"test\"}, 13e-17, null ]"),
            JsonElement::Array(a),
            {
                assert_eq!(4, a.len());
                unwrap_json_element!(a.get(0).unwrap(), JsonElement::Boolean(b), { assert!(b) });
                unwrap_json_element!(a.get(1).unwrap(), JsonElement::Object(o), {
                    assert_eq!(1, o.len());
                    unwrap_json_element!(o.get("name").unwrap(), JsonElement::Str(s), {
                        assert_eq!("test", s)
                    });
                });
                unwrap_json_element!(a.get(2).unwrap(), JsonElement::Number(n), {
                    assert_eq!(13e-17, *n)
                });
                unwrap_json_element!(a.get(3).unwrap(), JsonElement::Null, { assert!(true) });
            }
        );
    }

    #[test]
    fn should_parse_obj() {
        unwrap_json_element_result!(Parser::parse("{ }"), JsonElement::Object(o), {
            assert_eq!(0, o.len())
        });
        unwrap_json_element_result!(
            Parser::parse("{ \"test\": \"string\", \"num\": 156  }"),
            JsonElement::Object(o),
            {
                assert_eq!(2, o.len());
                unwrap_json_element!(o.get("test").unwrap(), JsonElement::Str(s), {
                    assert_eq!("string", s)
                });
                unwrap_json_element!(o.get("num").unwrap(), JsonElement::Number(n), {
                    assert_eq!(156.0, *n)
                });
            }
        );
    }
}
