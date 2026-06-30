# Redis Lite

A lightweight Redis server clone built from scratch in Rust, featuring an offset-driven, zero-allocation tracking decoder based on Arpit Bhayani's RESP wire protocol architecture.

## Current Progress

*   Configured a binary-safe TCP server listening on port `7379`.
*   Implemented full REAL-RESP decoding supporting Simple Strings, Errors, Integers, Bulk Strings, and Nested Multi-Bulk Arrays.
*   Added an exhaustive modular test suite for parser integrity and underflow/overflow bounds protection.

## How to Run

### 1. Start the Server
Run the engine in your main terminal environment:
```bash
cargo run
```

### 2. Test the Protocol Core
Because the server listens for raw Redis Serialization Protocol (RESP) wire streams, you can pipe direct multi-bulk arrays to evaluate the parser loop using Netcat (`nc`).

#### On macOS:
Use ANSI-C quoting alongside the closing stream flag (`-c`) to ensure the server flushes and renders your payload properly:
```bash
echo -n \$'*3\r\n:42\r\n+PING\r\n*1\r\n\$4\r\nRESP\r\n' | nc -c 127.0.0.1 7379
```

#### On Linux:
Use the standard write-termination flag (`-N`) to safely disconnect the channel on end-of-file:
```bash
echo -ne "*3\r\n:42\r\n+PING\r\n*1\r\n\$4\r\nRESP\r\n" | nc -N 127.0.0.1 7379
```

### Protocol Payload Trace Example
The test payload above decomposes inside the decoder as follows:
* `*3\r\n` $\rightarrow$ Array containing exactly 3 elements.
* `:42\r\n` $\rightarrow$ First element: Integer value `42`.
* `+PING\r\n` $\rightarrow$ Second element: Simple String `"PING"`.
* `*1\r\n$4\r\nRESP\r\n` $\rightarrow$ Third element: A nested sub-array containing a 4-byte Bulk String `"RESP"`.
