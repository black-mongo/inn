//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T11:23:35+08:00
//-------------------------------------------------------------------
use actix::prelude::*;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use inn_network::server::ProxyServer;
use inn_network::*;
use std::vec::Vec;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
#[actix_rt::test]
async fn ping() {
    // env_logger::init();
    let session = VisitorSession::default();
    let addr = session.start();
    let pong = addr.send(ToSession::Ping).await.unwrap();
    assert_eq!(pong, SessionReply::Pong);
    addr.send(ToSession::Stop).await.unwrap();
}
#[actix_rt::test]
async fn get_session_meta() {
    let session = VisitorSession::default();
    let addr = session.start();
    let meta = addr.send(ToSession::Meta).await.unwrap();
    assert_eq!(meta, SessionReply::Meta(SessionMeta(0)));
    addr.send(ToSession::Stop).await.unwrap();
}
#[actix_rt::test]
async fn stop_session() {
    let session = VisitorSession::default();
    let addr = session.start();
    let meta = addr.send(ToSession::Stop).await.unwrap();
    assert_eq!(meta, SessionReply::Ok);
}

struct Auth {
    id: String,
    pwd: String,
}
impl Default for Auth {
    fn default() -> Self {
        Auth {
            id: "admin".into(),
            pwd: "123456".into(),
        }
    }
}
impl From<Auth> for Vec<u8> {
    fn from(auth: Auth) -> Vec<u8> {
        let mut rs = vec![0x05, auth.id.len() as u8];
        rs.extend(auth.id.as_bytes());
        rs.push(auth.pwd.len() as u8);
        rs.extend(auth.pwd.as_bytes());
        rs
    }
}
#[allow(dead_code)]
enum AddressType {
    Ipv4,
    Domain,
}
struct Connection {
    t: AddressType,
    address: String,
    port: u16,
}
impl Default for Connection {
    fn default() -> Self {
        Connection {
            t: AddressType::Ipv4,
            address: "127.0.0.1".into(),
            port: 4555,
        }
    }
}
#[allow(dead_code)]
impl Connection {
    fn new(t: AddressType, address: &str, port: u16) -> Self {
        Connection {
            t,
            address: address.into(),
            port,
        }
    }
}

impl From<Connection> for Vec<u8> {
    fn from(conn: Connection) -> Vec<u8> {
        let mut rs: Vec<u8> = vec![0x05, 0x01, 0x00];
        // rs.push(0x05);
        // rs.push(0x01);
        // rs.push(0x00);
        match conn.t {
            AddressType::Domain => {
                rs.push(0x03);
                // domain len
                rs.push(conn.address.len() as u8);
                // domain string
                rs.extend(conn.address.as_bytes());
            }
            AddressType::Ipv4 => {
                rs.push(0x01);
                for ip in conn.address.split('.') {
                    rs.push(ip.parse::<u8>().unwrap());
                }
            }
        }
        let mut port = [0; 2];
        BigEndian::write_u16(&mut port, conn.port);
        rs.push(port[0]);
        rs.push(port[1]);

        rs
    }
}
#[ignore]
#[actix_rt::test]
async fn ct() {
    enum State {
        Undefined,
        Auth,
        Connection,
        Forward,
    }

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("trace"));
    let server = ProxyServer::default().start();
    let _ = NetWork
        .start(
            "127.0.0.1",
            4555,
            || {
                // Connect to server
                actix::spawn(async {
                    let mut stream = TcpStream::connect("127.0.0.1:4555").await.unwrap();
                    stream.write_all(&[5, 1, 0]).await.unwrap();
                    let mut state = State::Undefined;
                    loop {
                        let _ = stream.readable().await;
                        let mut buf = Vec::with_capacity(40);
                        match (&state, stream.try_read_buf(&mut buf)) {
                            (_, Ok(0)) => continue,
                            (State::Undefined, Ok(2)) => {
                                assert_eq!(buf, vec![5, 2]);
                                let rs: Vec<u8> = Auth::default().into();
                                stream.write_all(rs.as_slice()).await.unwrap();
                                state = State::Auth;
                            }
                            (State::Auth, Ok(2)) => {
                                assert_eq!(buf, vec![5, 0]);
                                let rs: Vec<u8> = Connection::default().into();
                                stream.write_all(rs.as_slice()).await.unwrap();
                                state = State::Connection;
                            }
                            (State::Connection, Ok(_)) => {
                                // assert_eq!(buf, vec![5, 2]);
                                state = State::Forward;
                                stream.write_all(&[5, 1, 0]).await.unwrap();
                            }
                            (State::Forward, Ok(3)) => {
                                assert_eq!(buf, vec![5, 2]);
                                break;
                            }
                            (_, Ok(_)) => {
                                continue;
                            }
                            (_, Err(ref e)) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            (_, Err(e)) => {
                                panic!("{:?}", e)
                            }
                        }
                    }
                });
            },
            server.clone(),
        )
        .await;
}
