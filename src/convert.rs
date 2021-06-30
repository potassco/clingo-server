use crate::utils::{ConfigurationResult, ServerError};
use clingo::{parse_term, Part, Symbol, TruthValue};
use serde_json::Value;

pub fn json_to_configuration_result(val: &Value) -> Result<ConfigurationResult, ServerError> {
    match val {
        Value::String(s) => Ok(ConfigurationResult::Value(s.clone())),
        Value::Null => Err(ServerError::InternalError(
            "Could not parse configuration data".to_string(),
        )),
        Value::Bool(_) => Err(ServerError::InternalError(
            "Could not parse configuration data".to_string(),
        )),
        Value::Number(_) => Err(ServerError::InternalError(
            "Could not parse configuration data".to_string(),
        )),
        Value::Array(a) => {
            let mut arr = Vec::with_capacity(a.len());
            for val in a {
                let x = json_to_configuration_result(val)?;
                arr.push(x)
            }
            Ok(ConfigurationResult::Array(arr))
        }
        Value::Object(m) => {
            let mut arr = Vec::with_capacity(m.len());
            for (e, val) in m {
                let x = json_to_configuration_result(val)?;
                arr.push((e.clone(), x))
            }
            Ok(ConfigurationResult::Map(arr))
        }
    }
}
pub fn json_to_symbol(val: &Value) -> Result<Symbol, ServerError> {
    match val {
        Value::String(s) => {
            let sym = clingo::parse_term(s)?;
            Ok(sym)
        }
        _ => Err(ServerError::InternalError(
            "Could not parse symbol data".to_string(),
        )),
    }
}
fn json_to_symbol_array(val: &Value) -> Result<Vec<Symbol>, ServerError> {
    match val {
        Value::Array(a) => {
            let mut arr = Vec::with_capacity(a.len());
            for val in a {
                let x = json_to_symbol(val)?;
                arr.push(x)
            }
            Ok(arr)
        }
        _ => Err(ServerError::InternalError(
            "Could not parse parts data".to_string(),
        )),
    }
}

pub fn json_to_parts(val: &Value) -> Result<Vec<Part>, ServerError> {
    match val {
        Value::Object(m) => {
            let mut parts = Vec::with_capacity(m.len());
            for (e, val) in m {
                let x = json_to_symbol_array(val)?;
                let part = Part::new(e, x)?;
                parts.push(part)
            }
            Ok(parts)
        }
        _ => Err(ServerError::InternalError(
            "Could not parse parts data".to_string(),
        )),
    }
}
pub fn json_to_assignment(val: &Value) -> Result<(Symbol, TruthValue), ServerError> {
    let parse_error = || ServerError::InternalError("Could not parse assignment data".to_string());
    match val {
        Value::Object(m) => {
            let val = m.get("literal").ok_or_else(parse_error)?;

            let symbol = match val {
                Value::String(e) => parse_term(e)?,
                _ => {
                    return Err(ServerError::InternalError(
                        "Could not parse assignment data".to_string(),
                    ))
                }
            };
            let val = m.get("truth_value").ok_or_else(parse_error)?;

            let truth_value = match val {
                Value::String(e) => match e.as_str() {
                    "True" => Ok(TruthValue::True),
                    "False" => Ok(TruthValue::False),
                    "Free" => Ok(TruthValue::Free),
                    _ => Err(parse_error()),
                },
                _ => Err(parse_error()),
            }?;
            Ok((symbol, truth_value))
        }
        _ => Err(parse_error()),
    }
}
pub fn json_to_assumptions(val: &Value) -> Result<Vec<(clingo::Symbol, bool)>, ServerError> {
    match val {
        Value::Array(a) => {
            let mut arr = Vec::with_capacity(a.len());
            for val in a {
                let val = match val {
                    Value::Array(a) => {
                        let name = match a.get(0) {
                            Some(Value::String(s)) => s,
                            _ => {
                                return Err(ServerError::InternalError(
                                    "Could not parse assumptions data".to_string(),
                                ))
                            }
                        };
                        let sym = clingo::parse_term(&name)?;

                        let sign = match a.get(1) {
                            Some(Value::Bool(b)) => *b,
                            _ => {
                                return Err(ServerError::InternalError(
                                    "Could not parse assumptions data".to_string(),
                                ))
                            }
                        };
                        (sym, sign)
                    }
                    _ => {
                        return Err(ServerError::InternalError(
                            "Could not parse assumptions data".to_string(),
                        ))
                    }
                };
                arr.push(val)
            }
            Ok(arr)
        }
        _ => Err(ServerError::InternalError(
            "Could not parse assumptions data".to_string(),
        )),
    }
}
