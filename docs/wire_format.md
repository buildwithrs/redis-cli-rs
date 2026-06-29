## 3. RESP Wire Format — Universal Rules

Every RESP frame — request or reply, RESP2 or RESP3 — follows the same three rules:

1. **Type byte first.** The first byte of a frame identifies its type (`*`, `$`, `+`, `-`, `:`, `%`, `~`, `>`, `,`, `#`, `_`, `(`, `=`).
2. **CRLF (`\r\n`) terminator.** Each header line and each data segment is terminated by the two-byte sequence `0x0D 0x0A`. The terminator is part of the protocol — it is what the client uses to know a line is complete.
3. **Length-prefixed where it makes sense.** Bulk strings, arrays, and most aggregate types carry an explicit length in the header so the reader knows exactly how many bytes to consume. This avoids fragile newline scanning of payload data.

Lengths are decimal ASCII digits, parsed as base-10 integers. A bulk string of length `-1` represents a **null bulk reply** (a `nil` value in Redis).

## 4. RESP2 — Reply Types

RESP2 defines five reply types. Clients that don't issue `HELLO 3` will receive only these.

### 4.1 Simple String — `+`

A non-binary status reply, e.g. `+OK\r\n`.

- Used for `PING` (`+PONG`), `SET` on success (`+OK`), and similar.
- Cannot contain `\r` or `\n` — those are reserved as frame delimiters.
- Limited to short, human-readable text.

### 4.2 Error — `-`

A reply that signals failure, e.g. `-ERR unknown command 'FOO'\r\n`.

- The first word after the `-` is conventionally the error class (`ERR`, `WRONGTYPE`, `OOM`, `NOAUTH`, `MOVED 1234 10.0.0.1:6379`, `ASK ...`, `LOADING`, `BUSY`, `MASTERDOWN`, etc.). Clients use this prefix to drive retry / failover logic.
- `MOVED` and `ASK` errors drive Redis Cluster client redirection.

### 4.3 Integer — `:`

A 64-bit signed integer in ASCII, e.g. `:1000\r\n` or `:-5\r\n`.

- Returned by `INCR`, `DBSIZE`, `EXISTS`, the `ZADD`/`SADD` count reply, and similar commands.
- Range is the platform's `long long` (typically -2^63 .. 2^63-1).

### 4.4 Bulk String — `$`

A length-prefixed binary-safe string. The header declares the byte count, then exactly that many bytes follow, then `\r\n`.

```
$6\r\nfoobar\r\n
```

Breakdown:
- `$6` — six bytes of payload follow.
- `foobar` — the payload itself (may be any byte, including `\r` or `\n`, because the length is explicit).
- `\r\n` — terminator after the payload.

A **null bulk string** is `$ -1\r\n`. It is *not* an empty string; it is the protocol-level equivalent of Redis `nil` and is what `GET` returns for a missing key.

A bulk string of length 0 is `$0\r\n\r\n` — an empty string, which is *not* the same as nil.

### 4.5 Array — `*`

A length-prefixed sequence of reply frames, possibly of mixed types.

```
*3\r\n$3\r\nSET\r\n$5\r\nmykey\r\n$7\r\nmyvalue\r\n
```

- `*3` — three elements follow.
- Each element is a complete RESP2 frame in turn (so an array of arrays, an array containing errors, etc., are all valid).

A **null array** is `*-1\r\n` (used by `LRANGE` when the key doesn't exist). An **empty array** is `*0\r\n`.

Arrays are how multi-bulk replies are delivered (`MGET`, `LRANGE`, `HGETALL`, etc.) and how Pub/Sub messages are framed under RESP2.