//! Rust values to SQLite values.

/// A value as SQLite stores it, so generated code names it through forgr.
pub use rusqlite::types::Value;

/// A Rust value a setter takes for a column whose Rust type is `T`: text for a
/// TEXT column, an integer for an INTEGER column, a float for a REAL column and
/// bytes for a BLOB column; a column of any other type takes any [`Value`].
pub trait IntoCell<T> {
    /// The value as SQLite stores it.
    fn into_cell(self) -> Value;
}

impl IntoCell<String> for String {
    fn into_cell(self) -> Value {
        Value::Text(self)
    }
}

impl IntoCell<String> for &str {
    fn into_cell(self) -> Value {
        Value::Text(self.to_owned())
    }
}

impl IntoCell<String> for &String {
    fn into_cell(self) -> Value {
        Value::Text(self.clone())
    }
}

impl IntoCell<i64> for i64 {
    fn into_cell(self) -> Value {
        Value::Integer(self)
    }
}

impl IntoCell<f64> for f64 {
    fn into_cell(self) -> Value {
        Value::Real(self)
    }
}

impl IntoCell<Vec<u8>> for Vec<u8> {
    fn into_cell(self) -> Value {
        Value::Blob(self)
    }
}

impl IntoCell<Vec<u8>> for &[u8] {
    fn into_cell(self) -> Value {
        Value::Blob(self.to_vec())
    }
}

impl IntoCell<Value> for Value {
    fn into_cell(self) -> Value {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cell<T>(value: impl IntoCell<T>) -> Value {
        value.into_cell()
    }

    #[test]
    fn each_rust_value_becomes_its_sqlite_value() {
        assert_eq!(cell::<String>("a"), Value::Text("a".into()));
        assert_eq!(cell::<String>(String::from("b")), Value::Text("b".into()));
        assert_eq!(cell::<i64>(3), Value::Integer(3));
        assert_eq!(cell::<f64>(0.5), Value::Real(0.5));
        assert_eq!(cell::<Vec<u8>>(&[1u8, 2][..]), Value::Blob(vec![1, 2]));
    }
}
