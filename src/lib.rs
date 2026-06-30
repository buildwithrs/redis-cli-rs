/// encode commands init by client, and decode result from server
pub mod codec;

/// client errors
pub mod errors;

/// Redis Values
/*
1. Simple String — `+`
2. Error — `-`
3. Integer — `:`
4. Bulk String — `$`
5. Array — `*`
*/
#[derive(Debug, PartialEq, Eq)]
pub enum RedisValue {
    SimpleString(String),
    BulkString(String),
    Integer(i64),
    Err(String),
    Array(Vec<RedisValue>),
    Nil,
}

impl std::fmt::Display for RedisValue {
    /// Render a `RedisValue` in the same shape that `redis-cli` prints to its
    /// REPL: simple strings are bare, bulk strings are double-quoted, integers
    /// are tagged `(integer) N`, errors are tagged `(error) msg`, empty arrays
    /// become `(empty array or set)`, and non-empty arrays are one-per-line.
    /// Only the top-level array gets the `N) ` prefix — a nested array just
    /// emits each of its elements on its own line, so parent numbering stays
    /// unambiguous.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write_cli(self, f, true)
    }
}

/// Internal helper. `top_level` decides whether an array emits the `N) `
/// prefix; scalars render identically either way.
fn write_cli(
    v: &RedisValue,
    f: &mut std::fmt::Formatter<'_>,
    top_level: bool,
) -> std::fmt::Result {
    match v {
        RedisValue::Nil => f.write_str("nil"),
        RedisValue::SimpleString(s) => f.write_str(s),
        RedisValue::BulkString(s) => write!(f, "\"{}\"", s),
        RedisValue::Integer(n) => write!(f, "(integer) {}", n),
        RedisValue::Err(s) => write!(f, "(error) {}", s),
        RedisValue::Array(arr) => {
            if arr.is_empty() {
                return f.write_str("(empty array or set)");
            }
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                if top_level {
                    write!(f, "{}) ", i + 1)?;
                }
                write_cli(item, f, false)?;
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RedisValue;

    #[test]
    fn display_simple_string_is_bare() {
        assert_eq!(
            format!("{}", RedisValue::SimpleString("OK".into())),
            "OK"
        );
    }

    #[test]
    fn display_bulk_string_is_quoted() {
        assert_eq!(
            format!("{}", RedisValue::BulkString("bar".into())),
            "\"bar\""
        );
    }

    #[test]
    fn display_integer_has_tag() {
        assert_eq!(format!("{}", RedisValue::Integer(42)), "(integer) 42");
        assert_eq!(format!("{}", RedisValue::Integer(-1)), "(integer) -1");
    }

    #[test]
    fn display_error_has_tag() {
        assert_eq!(
            format!("{}", RedisValue::Err("WRONGTYPE Operation against a key".into())),
            "(error) WRONGTYPE Operation against a key"
        );
    }

    #[test]
    fn display_empty_array() {
        assert_eq!(
            format!("{}", RedisValue::Array(vec![])),
            "(empty array or set)"
        );
    }

    #[test]
    fn display_array_of_bulk_strings_numbers_each_line() {
        let v = RedisValue::Array(vec![
            RedisValue::BulkString("first".into()),
            RedisValue::BulkString("second".into()),
        ]);
        assert_eq!(format!("{}", v), "1) \"first\"\n2) \"second\"");
    }

    #[test]
    fn display_array_of_mixed_types() {
        let v = RedisValue::Array(vec![
            RedisValue::BulkString("name".into()),
            RedisValue::BulkString("Alice".into()),
            RedisValue::Integer(30),
        ]);
        assert_eq!(
            format!("{}", v),
            "1) \"name\"\n2) \"Alice\"\n3) (integer) 30"
        );
    }

    #[test]
    fn display_nested_array_does_not_restart_prefix() {
        // Top-level array is prefixed `N) `, the inner array just lays its
        // elements on successive lines so we don't end up with two `2)`s.
        let v = RedisValue::Array(vec![
            RedisValue::Array(vec![RedisValue::Integer(1), RedisValue::Integer(2)]),
            RedisValue::SimpleString("OK".into()),
        ]);
        assert_eq!(format!("{}", v), "1) (integer) 1\n(integer) 2\n2) OK");
    }
}