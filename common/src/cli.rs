//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//  inn_cli tool using RESP(https://redis.io/docs/reference/protocol-spec/) to communication
//
// @end
// Created : 2022-04-18T23:22:42+08:00
//-------------------------------------------------------------------
use log::error;
use std::cmp::Ordering;
#[derive(Debug, Clone, PartialEq)]
pub enum Cli {
    // +OK\r\n
    SimpleString(String),
    // -Error Message\r\n
    Errors(String),
    // :10\r\n
    Integers(i64),
    // $6\r\nhello\r\n
    // An empty string
    // $0\r\n\r\n
    BulkString(String),
    // "$-1\r\n"
    NullBulkString,
    // *-1\r\n
    NullArrays,
    // An empty arrays
    // "*0\r\n"
    // [1,2,3,4,hello]
    // *5\r\n
    // :1\r\n
    // :2\r\n
    // :3\r\n
    // :4\r\n
    // $5\r\n
    // hello\r\n
    Arrays(Vec<Cli>),
}
impl Cli {
    fn parse(org: &str) -> (Cli, &str) {
        let mut str = org.splitn(2, "\r\n");
        let first = str.next();
        let second = str.next();
        if let Some(first) = first {
            match first.as_bytes()[0] {
                b'+' => (Cli::SimpleString(first[1..].into()), second.unwrap_or("")),
                b'-' => (Cli::Errors(first[1..].into()), second.unwrap_or("")),
                b':' => {
                    let i = &first[1..];
                    (
                        Cli::Integers(i.parse::<i64>().unwrap_or(0)),
                        second.unwrap_or(""),
                    )
                }
                b'$' => {
                    let len = &first[1..];
                    let len: i64 = len.parse::<i64>().unwrap_or(-10);
                    match len.cmp(&-1) {
                        Ordering::Equal => (Cli::NullBulkString, second.unwrap_or("")),
                        Ordering::Less => {
                            error!("Cli parse Bulk String Invalid len = {}, str = {}", len, org);
                            (Cli::NullBulkString, "")
                        }
                        Ordering::Greater => {
                            let second = second.unwrap_or("");
                            let second_len = second.len();
                            let len = len as usize;
                            if len > second_len || len + 2 > second_len {
                                error!(
                                    "Cli parse Bulk String Invalid len = {}, str = {}",
                                    len, org
                                );
                                (Cli::NullBulkString, "")
                            } else {
                                (
                                    Cli::BulkString(second[..len as usize].into()),
                                    second[len as usize + 2..].into(),
                                )
                            }
                        }
                    }
                }
                b'*' => {
                    let len = &first[1..];
                    let len: i64 = len.parse::<i64>().unwrap_or(-10);
                    match len.cmp(&-1) {
                        Ordering::Equal => (Cli::NullArrays, second.unwrap_or("")),
                        Ordering::Less => {
                            error!("Cli parse Arrays Invalid len = {}, str = {}", len, org);
                            (Cli::NullArrays, "")
                        }
                        Ordering::Greater => {
                            let mut arrays = vec![];
                            let mut remain = second.unwrap_or("");
                            for _i in 0..len {
                                if !remain.is_empty() {
                                    let (cli, other) = Cli::parse(remain);
                                    remain = other;
                                    arrays.push(cli);
                                }
                            }
                            (Cli::Arrays(arrays), remain)
                        }
                    }
                }
                _ => {
                    error!("Cli parse Invalid {}", org);
                    (Cli::NullArrays, "")
                }
            }
        } else {
            error!("Cli parse Invalid {}", org);
            (Cli::NullArrays, "")
        }
    }
}
impl From<String> for Cli {
    fn from(str: String) -> Self {
        let (cli, _) = Cli::parse(&str);
        cli
    }
}
impl From<Cli> for String {
    fn from(cli: Cli) -> Self {
        match cli {
            Cli::SimpleString(simple) => format!("+{}\r\n", simple),
            Cli::Errors(errors) => format!("-{}\r\n", errors),
            Cli::Integers(int) => format!(":{}\r\n", int),
            Cli::BulkString(bulk) => format!("${}\r\n{}\r\n", bulk.len(), bulk),
            Cli::NullBulkString => "$-1\r\n".into(),
            Cli::NullArrays => "*-1\r\n".into(),
            Cli::Arrays(arrays) if arrays.is_empty() => "*0\r\n".into(),
            Cli::Arrays(arrays) => {
                let mut rs = format!("*{}\r\n", arrays.len());
                for row in arrays {
                    let str: String = row.into();
                    rs += &str;
                }
                rs
            }
        }
    }
}