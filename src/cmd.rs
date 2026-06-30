use std::error::Error;

use crate::resp::RespValue;

#[derive(Debug)]
pub struct RedisCmd {
    pub name: String,
    pub args: Vec<String>,
}

impl RedisCmd {
    pub fn from_resp(value: RespValue) -> Result<RedisCmd, Box<dyn Error>> {
        if let RespValue::Array(elements) = value {
            if elements.is_empty() {
                return Err("Empty command array".into());
            }
            let mut tokens = Vec::new();
            for ele in elements {
                match ele {
                    RespValue::BulkString(s) | RespValue::SimpleString(s) => tokens.push(s),
                    RespValue::Integer(i) => tokens.push(i.to_string()),
                    _ => return Err("Invalid token type inside command array".into()),
                }
            }
            let name = tokens[0].to_uppercase();
            let args = tokens[1..].to_vec();
            Ok(RedisCmd { name, args })
        } else {
            Err("Redis client commands must be passed inside an Array payload".into())
        }
    }

    pub fn eval_and_respond(cmd: RedisCmd) -> Vec<u8> {
        match cmd.name.as_str() {
            "PING" => Self::eval_ping(cmd.args),
            _ => Self::encode_error(&format!("ERR unknown command '{}'", cmd.name)),
        }
    }

    fn eval_ping(args: Vec<String>) -> Vec<u8> {
        match args.len() {
            0 => Self::encode_simple_string("PONG"),
            1 => Self::encode_bulk_string(&args[0]),
            _ => Self::encode_error("ERR wrong number of arguments for 'ping' command"),
        }
    }

    fn encode_simple_string(s: &str) -> Vec<u8> {
        format!("+{}\r\n", s).into_bytes()
    }

    fn encode_error(msg: &str) -> Vec<u8> {
        format!("-{}\r\n", msg).into_bytes()
    }

    fn encode_bulk_string(s: &str) -> Vec<u8> {
        format!("${}\r\n{}\r\n", s.len(), s).into_bytes()
    }
}
