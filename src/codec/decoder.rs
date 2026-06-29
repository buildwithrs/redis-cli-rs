use tokio_util::bytes::{Buf, BytesMut};

use crate::{RedisValue, errors::CliErrors};

pub fn decode_cmd(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
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
    let tm = bs[size+1];
    let tm1 = bs[size+2];
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

pub fn decode_array(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_error(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}
