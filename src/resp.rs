use std::error::Error;

#[derive(Debug, PartialEq)]
pub enum RespValue {
    SimpleString(String),
    Error(String),
    Integer(i64),
    BulkString(String),
    Array(Vec<RespValue>),
}

pub fn decode(data: &[u8]) -> Result<RespValue, Box<dyn Error>> {
    if data.is_empty() {
        return Err("Empty data".to_string().into());
    }
    let (value, _) = decode_one(data)?;
    Ok(value)
}

fn decode_one(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    match data[0] {
        b'+' => read_simple_string(data),
        b'-' => read_error_data(data),
        b':' => read_integer(data),
        b'$' => read_bulk_string(data),
        b'*' => read_array(data),
        _ => Err(format!("Unknown RESP identifier: {}", data[0] as char).into()),
    }
}

fn read_simple_string(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    let mut pos = 1;
    while pos < data.len() {
        if data[pos] == b'\r' {
            let s = std::str::from_utf8(&data[1..pos])?.to_string();
            return Ok((RespValue::SimpleString(s), pos + 2));
        }
        pos += 1;
    }
    Err("Unterminated simple string".into())
}

fn read_error_data(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    let mut pos = 1; // Skip '-'
    while pos < data.len() {
        if data[pos] == b'\r' {
            if data[pos] < b'0' || data[pos] > b'9' {
                return Err(
                    format!("Expected numeric digit, found raw byte: {}", data[pos]).into(),
                );
            }
            let s = std::str::from_utf8(&data[1..pos])?.to_string();
            return Ok((RespValue::Error(s), pos + 2));
        }
        pos += 1;
    }
    Err("Unterminated error string".into())
}

fn read_integer(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    let mut pos = 1;
    let mut number: i64 = 0;
    while pos < data.len() && data[pos] != b'\r' {
        if data[pos] < b'0' || data[pos] > b'9' {
            return Err(format!("Expected numeric digit, found raw byte: {}", data[pos]).into());
        }
        number = number * 10 + (data[pos] - b'0') as i64;
        pos += 1;
    }
    Ok((RespValue::Integer(number), pos + 2))
}

fn read_bulk_string(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    let mut pos = 1;
    let mut number: usize = 0;
    while pos < data.len() && data[pos] != b'\r' {
        if data[pos] < b'0' || data[pos] > b'9' {
            return Err(format!("Expected numeric digit, found raw byte: {}", data[pos]).into());
        }
        number = number * 10 + (data[pos] - b'0') as usize;
        pos += 1;
    }
    pos += 2;
    let start = pos;
    let end = start + number;
    if end > data.len() {
        return Err("Bulk string length out of bounds".into());
    }
    let s = std::str::from_utf8(&data[start..end])?.to_string();
    Ok((RespValue::BulkString(s), end + 2))
}

fn read_array(data: &[u8]) -> Result<(RespValue, usize), Box<dyn Error>> {
    let mut pos = 1;
    let mut count: usize = 0;
    while pos < data.len() && data[pos] != b'\r' {
        if data[pos] < b'0' || data[pos] > b'9' {
            return Err(format!("Expected numeric digit, found raw byte: {}", data[pos]).into());
        }
        count = count * 10 + (data[pos] - b'0') as usize;
        pos += 1;
    }
    pos += 2;
    let mut elements = Vec::with_capacity(count);
    for _ in 0..count {
        let (element, consumed) = decode_one(&data[pos..])?;
        elements.push(element);
        pos += consumed;
    }
    Ok((RespValue::Array(elements), pos))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_empty_input() {
        let input = b"";
        let result = decode(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Empty data");
    }

    #[test]
    fn test_simple_string() {
        let input = b"+OK\r\n";
        let result = decode(input).unwrap();
        assert_eq!(result, RespValue::SimpleString("OK".to_string()));
    }

    #[test]
    fn test_decode_error() {
        let data = b"-ERR unknown command\r\n";
        let result = decode(data).unwrap();
        assert_eq!(result, RespValue::Error("ERR unknown command".to_string()));
    }

    #[test]
    fn test_decode_integer() {
        let data = b":1000\r\n";
        let result = decode(data).unwrap();
        assert_eq!(result, RespValue::Integer(1000));
        let data_zero = b":0\r\n";
        assert_eq!(decode(data_zero).unwrap(), RespValue::Integer(0));
    }

    #[test]
    fn test_decode_bulk_string() {
        let data = b"$5\r\nhello\r\n";
        let result = decode(data).unwrap();
        assert_eq!(result, RespValue::BulkString("hello".to_string()));
    }

    #[test]
    fn test_decode_bulk_string_out_of_bounds() {
        let data = b"$10\r\nhello\r\n";
        let result = decode(data);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_array() {
        let data = b"*2\r\n$5\r\nhello\r\n$5\r\nworld\r\n";
        let result = decode(data).unwrap();

        let expected = RespValue::Array(vec![
            RespValue::BulkString("hello".to_string()),
            RespValue::BulkString("world".to_string()),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_decode_nested_mixed_array() {
        let data = b"*3\r\n:42\r\n+PING\r\n*1\r\n$4\r\nRESP\r\n";
        let result = decode(data).unwrap();

        let expected = RespValue::Array(vec![
            RespValue::Integer(42),
            RespValue::SimpleString("PING".to_string()),
            RespValue::Array(vec![RespValue::BulkString("RESP".to_string())]),
        ]);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_unknown_identifier() {
        let data = b"xInvalid\r\n";
        let result = decode(data);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unknown RESP identifier")
        );
    }
}
