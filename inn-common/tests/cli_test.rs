//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-19T01:08:15+08:00
//-------------------------------------------------------------------
use inn_common::cli::Cli;
#[test]
fn test_simplestring_to_cli() {
    let cli: Cli = String::from("+OK\r\n").into();
    assert_eq!(cli, Cli::SimpleString("OK".into()));
}
#[test]
fn test_errors_to_cli() {
    let cli: Cli = String::from("-Errors\r\n").into();
    assert_eq!(cli, Cli::Errors("Errors".into()));
}
#[test]
fn test_integers_to_cli() {
    let cli: Cli = String::from(":10\r\n").into();
    assert_eq!(cli, Cli::Integers(10));
}
#[test]
fn test_bulkstring_to_cli() {
    let cli: Cli = String::from("$5\r\nhello\r\n").into();
    assert_eq!(cli, Cli::BulkString("hello".into()));
}
#[test]
fn test_empty_bulkstring_to_cli() {
    let cli: Cli = String::from("$0\r\n\r\n").into();
    assert_eq!(cli, Cli::BulkString("".into()));
}
#[test]
fn test_null_bulkstring_to_cli() {
    let cli: Cli = String::from("$-1\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
}
#[test]
fn test_arrays_to_cli() {
    let cli: Cli = String::from("*3\r\n+OK\r\n-Errors\r\n:10\r\n").into();
    assert_eq!(
        cli,
        Cli::Arrays(vec![
            Cli::SimpleString("OK".into()),
            Cli::Errors("Errors".into()),
            Cli::Integers(10)
        ])
    );
}
#[test]
fn test_nested_arrays_to_cli() {
    let cli: Cli = String::from("*5\r\n*2\r\n:5\r\n*-1\r\n*0\r\n+OK\r\n-Errors\r\n:10\r\n").into();
    assert_eq!(
        cli,
        Cli::Arrays(vec![
            Cli::Arrays(vec![Cli::Integers(5), Cli::NullArrays]),
            Cli::Arrays(vec![]),
            Cli::SimpleString("OK".into()),
            Cli::Errors("Errors".into()),
            Cli::Integers(10)
        ])
    );
}
#[test]
fn test_empty_and_null_buckstring_in_arrays_to_cli() {
    let cli: Cli = String::from("*4\r\n$0\r\n\r\n-Errors\r\n:10\r\n$-1\r\n\r\n").into();
    assert_eq!(
        cli,
        Cli::Arrays(vec![
            Cli::BulkString("".into()),
            Cli::Errors("Errors".into()),
            Cli::Integers(10),
            Cli::NullBulkString
        ])
    );
}
#[test]
fn test_empty_arrays_to_cli() {
    let cli: Cli = String::from("*0\r\n").into();
    assert_eq!(cli, Cli::Arrays(vec![]));
}
#[test]
fn test_null_arrays_to_cli() {
    let cli: Cli = String::from("*-1\r\n").into();
    assert_eq!(cli, Cli::NullArrays);
}
#[test]
fn test_cli_to_simplestring() {
    let str: String = Cli::SimpleString("OK".into()).into();
    assert_eq!("+OK\r\n", &str);
}
#[test]
fn test_cli_to_erros() {
    let str: String = Cli::Errors("Errors".into()).into();
    assert_eq!("-Errors\r\n", &str);
}
#[test]
fn test_cli_to_integers() {
    let str: String = Cli::Integers(10).into();
    assert_eq!(":10\r\n", &str);
}
#[test]
fn test_cli_to_bulk_string() {
    let str: String = Cli::BulkString("10".into()).into();
    assert_eq!("$2\r\n10\r\n", &str);
}
#[test]
fn test_cli_to_empty_bulk_string() {
    let str: String = Cli::BulkString("".into()).into();
    assert_eq!("$0\r\n\r\n", &str);
}
#[test]
fn test_cli_to_null_bulk_string() {
    let str: String = Cli::NullBulkString.into();
    assert_eq!("$-1\r\n", &str);
}
#[test]
fn test_cli_to_null_arrays() {
    let str: String = Cli::NullArrays.into();
    assert_eq!("*-1\r\n", &str);
}
#[test]
fn test_cli_to_empty_arrays() {
    let str: String = Cli::Arrays(vec![]).into();
    assert_eq!("*0\r\n", &str);
}
#[test]
fn test_cli_to_arrays() {
    let str: String = Cli::Arrays(vec![
        Cli::NullBulkString,
        Cli::BulkString("".into()),
        Cli::Arrays(vec![
            Cli::Integers(1),
            Cli::NullArrays,
            Cli::Arrays(vec![]),
            Cli::Errors("error".into()),
        ]),
        Cli::SimpleString("OK".into()),
        Cli::BulkString("YES".into()),
    ])
    .into();
    assert_eq!(
        "*5\r\n$-1\r\n$0\r\n\r\n*4\r\n:1\r\n*-1\r\n*0\r\n-error\r\n+OK\r\n$3\r\nYES\r\n",
        &str
    );
}
#[test]
fn test_invalid_string_to_cli() {
    let cli: Cli = String::from("$-10\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
    let cli: Cli = String::from("$-10\r\nhello\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
    let cli: Cli = String::from("$a\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
    let cli: Cli = String::from("$a\r\nhello\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
    let cli: Cli = String::from("$100\r\nhello\r\n").into();
    assert_eq!(cli, Cli::NullBulkString);
}
#[test]
fn test_invalid_arrays_to_cli() {
    let cli: Cli = String::from("*-10\r\n").into();
    assert_eq!(cli, Cli::NullArrays);
    let cli: Cli = String::from("*10\r\n").into();
    assert_eq!(cli, Cli::Arrays(vec![]));
    let cli: Cli = String::from("*a\r\n").into();
    assert_eq!(cli, Cli::NullArrays);
    let cli: Cli = String::from("*3\r\n:1\r\n:1\r\n").into();
    assert_eq!(cli, Cli::Arrays(vec![Cli::Integers(1), Cli::Integers(1)]));
}
