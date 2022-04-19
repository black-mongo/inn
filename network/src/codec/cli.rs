//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-19T10:13:32+08:00
//-------------------------------------------------------------------
use bytes::BytesMut;

use super::{VisitorDecoder, VisitorRequest};
use std::io::{Error, ErrorKind};
#[derive(Default)]
pub(crate) struct CliCodec {
    // default to 0
    pos: usize,
    //  CR = Carriage Return and LF = Line Feed
    // \r\n
    expected_crlf_count: usize,
    crlf_counter: usize,
}
impl CliCodec {
    fn parse_special(&mut self, buf: &[u8], cond: impl Fn(i64) -> usize) -> Result<(), Error> {
        let mut n = vec![];
        for i in 2..buf.len() {
            if buf[i - 1] == b'\r' && buf[i] == b'\n' {
                break;
            }
            n.push(buf[i - 1]);
        }
        if let Ok(n) = String::from_utf8(n) {
            if let Ok(n) = n.parse::<i64>() {
                if n == -1 || n == 0 {
                    self.expected_crlf_count += cond(n);
                } else if n < 0 {
                    return Err(Error::new(ErrorKind::Other, "Invalid data"));
                } else {
                    self.expected_crlf_count += cond(n);
                }
            }
        } else {
            return Err(Error::new(ErrorKind::Other, "Invalid utf8 data"));
        }
        Ok(())
    }
    fn parse_expected_crlf_count(&mut self, buf: &[u8]) -> Result<(), Error> {
        match buf[0] {
            b'+' => self.expected_crlf_count += 1,
            b'-' => self.expected_crlf_count += 1,
            b':' => self.expected_crlf_count += 1,
            b'$' => {
                let pos = self.pos;
                return self.parse_special(buf, |n| match (n, pos) {
                    (-1, 0) => 1,
                    (-1, _) => 0,
                    (_, 0) => 2,
                    (_, _) => 1,
                });
            }
            b'*' => {
                let pos = self.pos;
                return self.parse_special(buf, move |n| match (n, pos) {
                    (-1 | 0, 0) => 1,
                    (-1 | 0, _) => 0,
                    (_, 0) => n as usize + 1,
                    (_, _) => n as usize,
                });
            }
            _ => {
                return Err(Error::new(ErrorKind::Other, "Invalid RESP data"));
            }
        }
        Ok(())
    }
}
impl VisitorDecoder for CliCodec {
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VisitorRequest>, Error> {
        if src.len() < 4 {
            return Ok(None);
        }
        let buf = src.as_ref();
        // +ok\r\n
        for i in (self.pos + 1)..buf.len() {
            if buf[i - 1] == b'\r' && buf[i] == b'\n' {
                if self.pos == 0 || buf[self.pos] == b'*' || buf[self.pos] == b'$' {
                    let input = &buf[self.pos..];
                    if let Err(e) = self.parse_expected_crlf_count(input) {
                        return Err(e);
                    }
                }
                self.crlf_counter += 1;
                self.pos = i + 1;
                // Get a packet
                if self.crlf_counter == self.expected_crlf_count {
                    // split
                    let packet = src.split_to(i + 1);
                    let rs: common::cli::Cli = packet.to_vec().into();
                    self.pos = 0;
                    self.crlf_counter = 0;
                    self.expected_crlf_count = 0;
                    return Ok(Some(VisitorRequest::Cli(rs)));
                }
            }
        }
        Ok(None)
    }
}

#[cfg(test)]
mod test {
    use common::cli::Cli;

    use super::*;
    #[test]
    fn test_parse_expected_crlf_count_success() {
        check_ok(1, b"+OK\r\n", 1);
        check_ok(2, b"-Errors\r\n", 1);
        check_ok(3, b":10\r\n", 1);
        check_ok(4, b"$1\r\na\r\n", 2);
        check_ok(5, b"*1\r\n+OK\r\n", 2);
        check_ok(6, b"$0\r\n\r\n", 2);
        check_ok(7, b"$-1\r\n", 1);
        check_ok(8, b"*2\r\n+OK\r\n-Errors\r\n", 3);
        check_ok(9, b"*-1\r\n", 1);
    }
    #[test]
    fn test_parse_expected_crlf_failure() {
        check_error(1, b"a,\r\n");
        check_error(2, b"$aa\r\n");
        check_error(3, b"*tt\r\n");
    }
    #[test]
    fn test_clicodec_decode() {
        check_decode(1, b"+OK\r\n", Cli::SimpleString("OK".into()));
        check_decode(2, b"-Errors\r\n", Cli::Errors("Errors".into()));
        check_decode(3, b":10\r\n", Cli::Integers(10));
        check_decode(4, b"$0\r\n\r\n", Cli::BulkString("".into()));
        check_decode(5, b"$-1\r\n", Cli::NullBulkString);
        check_decode(6, b"*-1\r\n", Cli::NullArrays);
        check_decode(7, b"*0\r\n", Cli::Arrays(vec![]));
        check_decode(
            8,
            b"*1\r\n+OK\r\n",
            Cli::Arrays(vec![Cli::SimpleString("OK".into())]),
        );
        check_decode(
            9,
            b"*3\r\n+OK\r\n*0\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::Arrays(vec![]),
                Cli::Integers(10),
            ]),
        );
        check_decode(
            10,
            b"*3\r\n+OK\r\n*-1\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::NullArrays,
                Cli::Integers(10),
            ]),
        );
        check_decode(
            11,
            b"*3\r\n+OK\r\n$0\r\n\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::BulkString("".into()),
                Cli::Integers(10),
            ]),
        );
        check_decode(
            12,
            b"*4\r\n+OK\r\n$-1\r\n:10\r\n*1\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::NullBulkString,
                Cli::Integers(10),
                Cli::Arrays(vec![Cli::Integers(10)]),
            ]),
        );
    }
    #[test]
    fn test_clidecode_none() {
        let mut codec = CliCodec::default();
        check_decode_none(1, &mut codec, b"*4\r\n+OK\r\n$-1\r\n:10\r\n*1\r\n:10\r");
        check_decode_ok(
            2,
            &mut codec,
            b"*4\r\n+OK\r\n$-1\r\n:10\r\n*1\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::NullBulkString,
                Cli::Integers(10),
                Cli::Arrays(vec![Cli::Integers(10)]),
            ]),
        );
        check_decode_ok(
            3,
            &mut codec,
            b"*4\r\n+OK\r\n$-1\r\n:10\r\n*1\r\n:10\r\n",
            Cli::Arrays(vec![
                Cli::SimpleString("OK".into()),
                Cli::NullBulkString,
                Cli::Integers(10),
                Cli::Arrays(vec![Cli::Integers(10)]),
            ]),
        );
    }
    fn check_decode_ok(
        line: usize,
        codec: &mut CliCodec,
        input: &[u8],
        expected: common::cli::Cli,
    ) {
        let mut bytes = BytesMut::from(input);
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!((line, rs), (line, Some(VisitorRequest::Cli(expected))));
    }
    fn check_decode_none(line: usize, codec: &mut CliCodec, input: &[u8]) {
        let mut bytes = BytesMut::from(input);
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!((line, rs), (line, None));
    }
    fn check_decode(line: usize, input: &[u8], expected: common::cli::Cli) {
        let mut codec = CliCodec::default();
        let mut bytes = BytesMut::from(input);
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!((line, rs), (line, Some(VisitorRequest::Cli(expected))));
    }
    fn check_ok(line: usize, input: &[u8], expected_crlf_count: usize) {
        let mut codec = CliCodec::default();
        if let Ok(()) = codec.parse_expected_crlf_count(input) {
            assert_eq!(
                vec![line, codec.expected_crlf_count],
                vec![line, expected_crlf_count]
            )
        } else {
            panic!("error")
        }
    }
    fn check_error(line: usize, input: &[u8]) {
        let mut codec = CliCodec::default();
        let _ = codec.parse_expected_crlf_count(input);
        assert_eq!(vec![line, codec.expected_crlf_count], vec![line, 0]);
    }
}
