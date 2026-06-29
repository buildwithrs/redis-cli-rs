use tokio_util::bytes::{BufMut, Bytes, BytesMut};

use crate::errors::CliErrors;


/*
Every Redis command is sent as a **RESP array of bulk strings** 
(this format is identical in RESP2 and RESP3; 
clients do not change how they encode requests). 
The array's length is the command's arity (command name + arguments), 
and each element is a bulk string with the argument bytes.
*/

pub fn encode_cmd(values: Vec<String>) -> Result<Bytes, CliErrors> {
    let len = values.len();
    let mut buf = BytesMut::new();
    buf.put_slice(format!("*{}\r\n", len).as_bytes());

    for val in values {
        buf.put_slice(format!("${}\r\n", val.len()).as_bytes());
        buf.put_slice(format!("{}\r\n", val).as_bytes());
    }

    Ok(buf.freeze())
}
