use tokio_util::bytes::{Buf, BytesMut};

use crate::{RedisValue, errors::CliErrors};

pub fn decode_redis_value(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    let prefix = bs.get_u8();
    match prefix {
        b'+' => decode_simple_string(bs),
        b':' => decode_integer(bs),
        b'$' => {
            let size = read_size(bs)?;
            decode_bulk_string(bs, size as usize)
        }
        b'-' => decode_error(bs),
        b'*' => {
            let size = read_size(bs)? as usize;
            decode_array(bs, size)
        }
        _ => Err(CliErrors::InvalidRedisValue(format!(
            "unknown redis data type: {}",
            prefix
        ))),
    }
}

pub fn decode_simple_string(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    let mut bss = Vec::new();
    loop {
        let e = bs.get_u8();
        if e == b'\r' {
            // expect \n
            let e1 = bs.get_u8();
            if e1 != b'\n' {
                return Err(CliErrors::InvalidRedisValue("expect \n".to_string()));
            }
            break;
        }

        bss.push(e);
    }

    let s = String::from_utf8_lossy(&bss);
    Ok(RedisValue::SimpleString(s.to_string()))
}

/// $n\r\n{n_bytes_data}\r\n
pub fn decode_bulk_string(bs: &mut BytesMut, size: usize) -> Result<RedisValue, CliErrors> {
    if bs.len() < size + 2 {
        return Err(CliErrors::InvalidRedisValue("bad bulk string".to_string()));
    }

    let bulk_bs = &bs[0..size];
    let tm = bs[size + 1];
    let tm1 = bs[size + 2];
    if tm != b'\r' && tm1 != b'\n' {
        return Err(CliErrors::InvalidRedisValue("bad bulk string".to_string()));
    }

    let s = String::from_utf8_lossy(&bulk_bs);
    Ok(RedisValue::BulkString(s.to_string()))
}

pub fn decode_integer(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    let mut bss = Vec::new();
    loop {
        let e = bs.get_u8();
        if e == b'\r' {
            // expect \n
            let e1 = bs.get_u8();
            if e1 != b'\n' {
                return Err(CliErrors::InvalidRedisValue("expect \n".to_string()));
            }
            break;
        }

        bss.push(e);
    }

    let n_s = String::from_utf8_lossy(&bss);
    let integer = n_s.parse::<i64>()?;
    Ok(RedisValue::Integer(integer))
}

pub fn decode_array(bs: &mut BytesMut, size: usize) -> Result<RedisValue, CliErrors> {
    let mut arr = vec![];
    for _ in 0..size {
        arr.push(decode_redis_value(bs)?);
    }

    Ok(RedisValue::Array(arr))
}

pub fn decode_error(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    let mut bss = Vec::new();
    loop {
        let e = bs.get_u8();
        if e == b'\r' {
            // expect \n
            let e1 = bs.get_u8();
            if e1 != b'\n' {
                return Err(CliErrors::InvalidRedisValue("expect \n".to_string()));
            }
            break;
        }

        bss.push(e);
    }

    let n_s = String::from_utf8_lossy(&bss);
    Ok(RedisValue::Err(n_s.to_string()))
}

fn read_size(bs: &mut BytesMut) -> Result<u64, CliErrors> {
    let mut bss = Vec::new();
    loop {
        let e = bs.get_u8();
        if e == b'\r' {
            // expect \n
            let e1 = bs.get_u8();
            if e1 != b'\n' {
                return Err(CliErrors::InvalidRedisValue("expect \n".to_string()));
            }
            break;
        }

        bss.push(e);
    }

    let n_s = String::from_utf8_lossy(&bss);
    Ok(n_s.parse::<u64>()?)
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::BytesMut;

    use super::decode_redis_value;
    use crate::RedisValue;
    use crate::errors::CliErrors;

    /// Build a `BytesMut` from a static byte slice.
    fn buf(input: &[u8]) -> BytesMut {
        let mut b = BytesMut::new();
        b.extend_from_slice(input);
        b
    }

    // ---------- simple string ----------

    #[test]
    fn decode_simple_string_ok() {
        let mut b = buf(b"+OK\r\n");
        let result = decode_redis_value(&mut b).unwrap();
        assert_eq!(result, RedisValue::SimpleString("OK".to_string()));
        assert!(b.is_empty(), "buffer should be fully consumed");
    }

    #[test]
    fn decode_simple_string_empty() {
        let mut b = buf(b"+\r\n");
        let result = decode_redis_value(&mut b).unwrap();
        assert_eq!(result, RedisValue::SimpleString(String::new()));
    }

    #[test]
    fn decode_simple_string_missing_lf_after_cr_returns_err() {
        let mut b = buf(b"+OK\rX");
        assert!(matches!(
            decode_redis_value(&mut b),
            Err(CliErrors::InvalidRedisValue(_))
        ));
    }

    // ---------- integer ----------

    #[test]
    fn decode_integer_positive() {
        let mut b = buf(b":1234\r\n");
        assert_eq!(
            decode_redis_value(&mut b).unwrap(),
            RedisValue::Integer(1234)
        );
    }

    #[test]
    fn decode_integer_negative() {
        let mut b = buf(b":-42\r\n");
        assert_eq!(
            decode_redis_value(&mut b).unwrap(),
            RedisValue::Integer(-42)
        );
    }

    #[test]
    fn decode_integer_zero() {
        let mut b = buf(b":0\r\n");
        assert_eq!(decode_redis_value(&mut b).unwrap(), RedisValue::Integer(0));
    }

    #[test]
    fn decode_integer_invalid_returns_err() {
        let mut b = buf(b":notanumber\r\n");
        assert!(matches!(
            decode_redis_value(&mut b),
            Err(CliErrors::InvalidRedisInteger(_))
        ));
    }

    // ---------- error ----------

    #[test]
    fn decode_error() {
        let mut b = buf(b"-WRONGTYPE Operation against a key holding the wrong kind of value\r\n");
        assert_eq!(
            decode_redis_value(&mut b).unwrap(),
            RedisValue::Err(
                "WRONGTYPE Operation against a key holding the wrong kind of value".to_string()
            )
        );
    }

    #[test]
    fn decode_error_empty_message() {
        let mut b = buf(b"-\r\n");
        assert_eq!(
            decode_redis_value(&mut b).unwrap(),
            RedisValue::Err(String::new())
        );
    }

    // ---------- array ----------

    #[test]
    fn decode_empty_array() {
        let mut b = buf(b"*0\r\n");
        assert_eq!(
            decode_redis_value(&mut b).unwrap(),
            RedisValue::Array(vec![])
        );
    }

    #[test]
    fn decode_array_of_integers() {
        let mut b = buf(b"*3\r\n:1\r\n:2\r\n:3\r\n");
        let expected = RedisValue::Array(vec![
            RedisValue::Integer(1),
            RedisValue::Integer(2),
            RedisValue::Integer(3),
        ]);
        assert_eq!(decode_redis_value(&mut b).unwrap(), expected);
    }

    #[test]
    fn decode_array_of_simple_strings_and_errors() {
        let mut b = buf(b"*3\r\n+OK\r\n-ERR boom\r\n+PONG\r\n");
        let expected = RedisValue::Array(vec![
            RedisValue::SimpleString("OK".to_string()),
            RedisValue::Err("ERR boom".to_string()),
            RedisValue::SimpleString("PONG".to_string()),
        ]);
        assert_eq!(decode_redis_value(&mut b).unwrap(), expected);
    }

    #[test]
    fn decode_nested_array() {
        // *2\r\n*1\r\n:99\r\n+OK\r\n
        let mut b = buf(b"*2\r\n*1\r\n:99\r\n+OK\r\n");
        let expected = RedisValue::Array(vec![
            RedisValue::Array(vec![RedisValue::Integer(99)]),
            RedisValue::SimpleString("OK".to_string()),
        ]);
        assert_eq!(decode_redis_value(&mut b).unwrap(), expected);
    }

    // ---------- unknown prefix ----------

    #[test]
    fn decode_unknown_prefix_returns_err() {
        let mut b = buf(b"?weird\r\n");
        assert!(matches!(
            decode_redis_value(&mut b),
            Err(CliErrors::InvalidRedisValue(msg)) if msg.contains("unknown redis data type")
        ));
    }
}
