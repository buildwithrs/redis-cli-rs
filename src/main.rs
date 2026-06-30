use redis_cli_rs::{codec::{decoder::decode_redis_value, encoder::encode_cmd, parse_cmd_to_strings}, errors::CliErrors};
use tokio::{
    io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader, Stdin}, net::TcpStream,
};
use tokio_util::codec::{BytesCodec, FramedRead, FramedWrite};
use futures::{StreamExt, SinkExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("A Redis Client Build with Rust");

    let addr = "127.0.0.1:6379";
    let stream = TcpStream::connect(addr).await?;

    let (reader, writer) = stream.into_split();
    let mut framed_reader = FramedRead::new(reader, BytesCodec::new());
    let mut framed_writer = FramedWrite::new(writer, BytesCodec::new());

    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);
    let mut stdout = io::stdout();
    loop {
        stdout.write_all(b"> ").await?;
        stdout.flush().await?;

        let (cmd, end) = read_line(&mut reader).await?;
        if end {
            break;
        }
        if cmd == "exit" {
            break;
        }

        let cmds = parse_cmd_to_strings(&cmd)?;
        println!("sending cmds: {:?} to redis server", cmds);

        let encode_cmd = encode_cmd(cmds)?;
        let _ = framed_writer.send(encode_cmd).await?;

        if let Some(val) = framed_reader.next().await {
            match val {
                Ok(mut bs) => {
                    let val = decode_redis_value(&mut bs)?;
                    println!("{}", val);
                }
                Err(e) => {
                    stdout.write_all(&e.to_string().into_bytes()).await?;
                }
            }
        }
    }

    Ok(())
}

async fn read_line(stdin: &mut BufReader<Stdin>) -> Result<(String, bool), CliErrors> {
    let mut line = String::new();
    let n = stdin.read_line(&mut line).await.expect("failed to read line");
    if n == 0 {
        return Ok(("".to_string(), true));
    }

    let cmd = line.trim_end_matches(|c| c == '\r' || c == '\n');
    if cmd.is_empty() {
        return Ok(("".to_string(), false));
    }

    println!("get cmd: {}", cmd);

    Ok((cmd.to_string(), false))
}
