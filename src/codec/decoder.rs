use tokio_util::bytes::BytesMut;

use crate::{RedisValue, errors::CliErrors};

pub fn decode_cmd(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_simple_string(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_bulk_string(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_integer(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_array(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}

pub fn decode_error(bs: &mut BytesMut) -> Result<RedisValue, CliErrors> {
    Ok(RedisValue::SimpleString("OK".to_string()))
}
