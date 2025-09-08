use snix_eval::{EvaluationResult, Value};

pub(crate) fn convert_expression_to_string(expression: EvaluationResult) -> Option<String> {
    match expression.value {
        None => Some("Null".to_string()),
        Some(Value::Null) => Some("Null".to_string()),
        Some(Value::Bool(bool)) => Some(bool.to_string()),
        Some(Value::Integer(integer)) => Some(integer.to_string()),
        Some(Value::Float(float)) => Some(float.to_string()),
        Some(Value::String(string)) => Some(string.to_string()),
        Some(Value::Path(_path)) => todo!("Convert the path data type to a String"),
        Some(Value::Attrs(_attrs)) => todo!("Convert the attrs data type to a String"),
        Some(Value::List(_list)) => todo!("Convert the list data type to a String"),
        Some(Value::Closure(_closure)) => todo!("Convert the closure data type to a String"),
        Some(Value::Builtin(_builtins)) => todo!("Convert the builtins data type to a String"),
        Some(Value::Thunk(_thunk)) => todo!("Convert the thunk data type to a String"),
        Some(Value::AttrNotFound) => todo!("Convert the attrNotFound data type to a String"),
        Some(Value::Blueprint(_blueprint)) => todo!("Convert the blueprint data type to a String"),
        Some(Value::DeferredUpvalue(_deferred)) => {
            todo!("Convert the deferred data type to a String")
        }
        Some(Value::UnresolvedPath(_unresolved)) => {
            todo!("Convert the unresolved path data type to a String")
        }
        Some(Value::FinaliseRequest(_finalise)) => {
            todo!("Convert the finalise request data type to a String")
        }
        Some(Value::Catchable(_catchable)) => todo!("Convert the catchable data type to a String"),
    }
}
