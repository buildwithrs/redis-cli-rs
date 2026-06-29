pub mod client; // the redis client

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
pub enum RedisValue {
    SimpleString(String),
    BulkString(String),
    Integer(i64),
    Err(String),
    Array(Vec<RedisValue>)
}