use tokio_util::bytes::{BufMut, Bytes, BytesMut};

use crate::errors::CliErrors;

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
