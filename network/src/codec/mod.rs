//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-15T16:32:29+08:00
//-------------------------------------------------------------------
use actix_codec::Decoder;
use actix_codec::Encoder;
use actix_web::web::BytesMut;
use byteorder::BigEndian;
use byteorder::ByteOrder;
use log::trace;
use std::io::{Error, ErrorKind};

use crate::server::ProxyServer;
pub mod forward;
pub struct VisitorCodec {
    state: State,
    proto: Proto,
}
impl Default for VisitorCodec {
    fn default() -> Self {
        VisitorCodec {
            state: State::Undefined,
            proto: Proto::Undefined,
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
enum State {
    Undefined,
    Greeting,
    Auth,
    Forward,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Proto {
    Undefined,
    Socks5,
}
#[derive(Debug, PartialEq)]
pub enum Cmd {
    Connection,
    Binding,
    AssociateUdp,
}
#[derive(Debug, PartialEq, Clone)]
pub enum T {
    IPv4,
    Domain,
}
#[derive(Debug, PartialEq, Clone)]
pub struct DstAddress {
    pub(crate) t: T,
    pub(crate) addr: String,
    pub(crate) port: u16,
}
impl DstAddress {
    pub fn new(t: T, addr: &str, port: u16) -> Self {
        DstAddress {
            t,
            addr: addr.into(),
            port,
        }
    }
}
impl DstAddress {
    fn port(&self) -> Vec<u8> {
        let mut rs = [0; 2];
        BigEndian::write_u16(&mut rs, self.port);
        rs.to_vec()
    }
    fn address(&self) -> Vec<u8> {
        let mut rs = vec![];
        match self.t {
            T::IPv4 => {
                rs.push(0x01);
                for row in self.addr.split('.') {
                    rs.push(row.parse::<u8>().unwrap());
                }
            }
            T::Domain => {
                rs.push(0x03);
                rs.push(self.addr.len() as u8);
                rs.extend(self.addr.as_bytes());
            }
        }
        rs
    }
}
impl From<DstAddress> for Vec<u8> {
    fn from(addr: DstAddress) -> Vec<u8> {
        let mut rs = vec![];
        rs.extend(addr.address());
        rs.extend(addr.port());
        rs
    }
}
#[derive(Debug, PartialEq)]
pub enum VisitorRequest {
    Greeting { proto: Proto, auth: Vec<u8> },
    Auth { id: String, pwd: String },
    Connection { cmd: Cmd, address: DstAddress },
    Forward(Vec<u8>),
}
#[derive(Debug, PartialEq)]
pub enum AuthChoice {
    NoAuth,
    UserNamePwd,
    NoAcceptable,
}
impl From<AuthChoice> for u8 {
    fn from(choice: AuthChoice) -> u8 {
        match choice {
            AuthChoice::NoAcceptable => 0xFF,
            AuthChoice::UserNamePwd => 0x02,
            AuthChoice::NoAuth => 0x00,
        }
    }
}
impl From<u8> for AuthChoice {
    fn from(v: u8) -> Self {
        match v {
            0x02 => AuthChoice::UserNamePwd,
            0x00 => AuthChoice::NoAuth,
            _ => AuthChoice::NoAcceptable,
        }
    }
}
#[derive(Debug, PartialEq, Clone)]
pub enum BindStatus {
    Granted,
    Failure,
    NotAllowedRuleSet,
    NetWorkUnReachable,
    HostUnReachable,
    ConnectionRefuse,
    TTLExpired,
    CommandNotSupported,
    AddressTypeNotSupported,
}
impl From<BindStatus> for u8 {
    fn from(status: BindStatus) -> u8 {
        match status {
            BindStatus::Granted => 0x00,
            BindStatus::Failure => 0x01,
            BindStatus::NotAllowedRuleSet => 0x02,
            BindStatus::NetWorkUnReachable => 0x03,
            BindStatus::HostUnReachable => 0x04,
            BindStatus::ConnectionRefuse => 0x05,
            BindStatus::TTLExpired => 0x06,
            BindStatus::CommandNotSupported => 0x07,
            BindStatus::AddressTypeNotSupported => 0x08,
        }
    }
}
#[derive(Debug, PartialEq)]
pub enum VisitorResponse {
    Choice(AuthChoice),
    AuthRespSuccess,
    AuthRespError,
    BindResp {
        status: BindStatus,
        address: Option<DstAddress>,
    },
    Forward(Vec<u8>),
}
impl From<VisitorResponse> for Vec<u8> {
    #[allow(clippy::vec_init_then_push)]
    fn from(resp: VisitorResponse) -> Vec<u8> {
        match resp {
            VisitorResponse::Choice(choice) => {
                let mut rs = vec![];
                rs.push(0x5);
                rs.push(choice.into());
                rs
            }
            VisitorResponse::AuthRespSuccess => {
                vec![0x5, 0x00]
            }
            VisitorResponse::AuthRespError => {
                vec![0x5, 0x01]
            }
            VisitorResponse::BindResp { status, address } => {
                let mut rs = vec![];
                rs.push(0x05);
                rs.push(status.clone().into());
                if status == BindStatus::Granted {
                    if let Some(address) = address {
                        rs.push(0x00);
                        let addr: Vec<u8> = address.into();
                        rs.extend(addr);
                    }
                }
                rs
            }
            VisitorResponse::Forward(data) => data,
        }
    }
}
impl Decoder for VisitorCodec {
    type Item = VisitorRequest;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!("Client data:{:?}", src.to_vec());
        if src.len() < 3 {
            return Ok(None);
        }
        match (&self.proto, &self.state) {
            (Proto::Undefined, State::Undefined) => {
                let buf = src.as_ref();
                if buf[0] == 0x05 {
                    let nauth = buf[1] as usize;
                    if src.len() < (nauth + 2) {
                        Ok(None)
                    } else {
                        let _ = src.split_to(2);
                        let buf = src.split_to(nauth as usize);
                        self.state = State::Auth;
                        match ProxyServer::auth_choice(&buf.to_vec()) {
                            AuthChoice::NoAcceptable => self.state = State::Greeting,
                            AuthChoice::UserNamePwd => self.state = State::Greeting,
                            AuthChoice::NoAuth => self.state = State::Auth,
                        }
                        self.proto = Proto::Socks5;
                        Ok(Some(VisitorRequest::Greeting {
                            proto: Proto::Socks5,
                            auth: buf.to_vec(),
                        }))
                    }
                } else {
                    let msg = format!("Invalid socks5 protocol");
                    Err(Error::new(ErrorKind::Other, msg))
                }
            }
            (Proto::Socks5, State::Greeting) => {
                // remove ver 0x05
                if src.len() < 2 {
                    return Ok(None);
                }
                let buf = src.as_ref();
                let id_len = buf[1] as usize;
                if src.len() < id_len + 3 {
                    return Ok(None);
                }
                let pwd_len = buf[id_len + 2] as usize;
                if src.len() < pwd_len + id_len + 3 {
                    return Ok(None);
                }
                let _ = src.split_to(2);
                let id = src.split_to(id_len);
                let _ = src.split_to(1);
                let pwd = src.split_to(pwd_len);
                self.state = State::Auth;
                Ok(Some(VisitorRequest::Auth {
                    id: String::from_utf8(id.to_vec()).unwrap(),
                    pwd: String::from_utf8(pwd.to_vec()).unwrap(),
                }))
            }
            (Proto::Socks5, State::Auth) => {
                // Client connection Request
                if src.len() < 5 {
                    return Ok(None);
                }
                let buf = src.as_ref();
                if buf[1] != 0x01 {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Client connection Request only support stream Connection",
                    ));
                }
                let addr_type = buf[3];
                match addr_type {
                    // IPV4
                    0x01 => {
                        if src.len() < 10 {
                            return Ok(None);
                        }
                        let _ = src.split_to(4);
                        let ip = src.split_to(4);
                        let port = BigEndian::read_u16(src.as_ref());
                        self.state = State::Forward;
                        let _ = src.split_to(2);
                        return Ok(Some(VisitorRequest::Connection {
                            cmd: Cmd::Connection,
                            address: DstAddress::new(
                                T::IPv4,
                                &format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3]),
                                port,
                            ),
                        }));
                    }
                    // Domain
                    0x03 => {
                        let name_len = buf[4] as usize;
                        if src.len() < name_len + 7 {
                            return Ok(None);
                        }
                        let _ = src.split_to(5);
                        let name = src.split_to(name_len);
                        let port = BigEndian::read_u16(src.as_ref());
                        let _ = src.split_to(2);
                        self.state = State::Forward;
                        if let Ok(name) = String::from_utf8(name.to_vec()) {
                            Ok(Some(VisitorRequest::Connection {
                                cmd: Cmd::Connection,
                                address: DstAddress::new(T::Domain, &name, port),
                            }))
                        } else {
                            Err(Error::new(
                                ErrorKind::Other,
                                "Client connection Request Domain invalid",
                            ))
                        }
                    }
                    _ => Err(Error::new(
                        ErrorKind::Other,
                        "Client connection Request only support IPv4 or Domain",
                    )),
                }
            }
            // forward
            (Proto::Socks5, State::Forward) => Ok(Some(VisitorRequest::Forward(
                src.split_to(src.len()).to_vec(),
            ))),
            _ => Ok(None),
        }
    }
}
impl Encoder<VisitorResponse> for VisitorCodec {
    type Error = Error;
    fn encode(&mut self, item: VisitorResponse, dst: &mut BytesMut) -> Result<(), Self::Error> {
        trace!("proto={:?}, VisitorResponse = {:?}", self.proto, item);
        let buf: Vec<u8> = item.into();
        dst.extend_from_slice(buf.as_slice());
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn visitor_response_encode() {
        check_visitor_response(VisitorResponse::AuthRespError, vec![0x05, 0x01]);
        check_visitor_response(VisitorResponse::AuthRespSuccess, vec![0x05, 0x00]);
        check_visitor_response(
            VisitorResponse::Choice(AuthChoice::NoAuth),
            vec![0x05, 0x00],
        );
        check_visitor_response(
            VisitorResponse::Choice(AuthChoice::UserNamePwd),
            vec![0x05, 0x02],
        );
        check_visitor_response(
            VisitorResponse::Choice(AuthChoice::NoAcceptable),
            vec![0x05, 0xFF],
        );
        check_visitor_response(
            VisitorResponse::BindResp {
                status: BindStatus::Failure,
                address: None,
            },
            vec![0x05, 0x01],
        );
        check_visitor_response(
            VisitorResponse::BindResp {
                status: BindStatus::Granted,
                address: Some(DstAddress::new(T::IPv4, "127.0.0.1", 80)),
            },
            vec![0x05, 0x00, 0x00, 0x01, 127, 0, 0, 1, 0, 80],
        );
        check_visitor_response(
            VisitorResponse::BindResp {
                status: BindStatus::Granted,
                address: Some(DstAddress::new(T::Domain, "abc", 80)),
            },
            vec![0x05, 0x00, 0x00, 0x03, 0x03, 97, 98, 99, 0, 80],
        );
        check_visitor_response(
            VisitorResponse::Forward(vec![1, 2, 3, 4, 5]),
            vec![1, 2, 3, 4, 5],
        );
    }
    fn check_visitor_response(resp: VisitorResponse, expected: Vec<u8>) {
        let rs: Vec<u8> = resp.into();
        assert_eq!(rs, expected);
    }
    #[test]
    fn socks5_decode() {
        env_logger::init();
        check_none(vec![5]);
        check_none(vec![5, 1]);
        check_none(vec![5, 2, 0]);
        check_greeting(
            vec![5, 1, 1],
            VisitorRequest::Greeting {
                proto: Proto::Socks5,
                auth: vec![1],
            },
            vec![],
        );
        check_greeting(
            vec![5, 2, 0, 1],
            VisitorRequest::Greeting {
                proto: Proto::Socks5,
                auth: vec![0, 1],
            },
            vec![],
        );
        check_greeting(
            vec![5, 2, 0, 1, 4],
            VisitorRequest::Greeting {
                proto: Proto::Socks5,
                auth: vec![0, 1],
            },
            vec![4],
        );
        check_auth_none(vec![5, 3, 97, 98, 99, 1], vec![5, 3, 97, 98, 99, 1]);
        check_auth_none(vec![5, 3, 97, 98, 99], vec![5, 3, 97, 98, 99]);
        check_auth_none(vec![5, 3, 97, 98], vec![5, 3, 97, 98]);
        check_auth_none(vec![5, 3], vec![5, 3]);
        check_auth_none(vec![5], vec![5]);
        check_auth_none(vec![], vec![]);
        // ID=abc,PWD=d
        check_auth(
            vec![5, 3, 97, 98, 99, 1, 100],
            VisitorRequest::Auth {
                id: "abc".into(),
                pwd: "d".into(),
            },
            vec![],
        );
        // ID=abc,PWD=d
        check_auth(
            vec![5, 3, 97, 98, 99, 1, 100, 200],
            VisitorRequest::Auth {
                id: "abc".into(),
                pwd: "d".into(),
            },
            vec![200],
        );
        check_connection_none(
            vec![5, 1, 0, 1, 192, 168, 1, 1, 0],
            vec![5, 1, 0, 1, 192, 168, 1, 1, 0],
        );
        check_connection_none(
            vec![5, 1, 0, 1, 192, 168, 1, 1],
            vec![5, 1, 0, 1, 192, 168, 1, 1],
        );
        check_connection_none(vec![5, 1, 0, 1, 192, 168], vec![5, 1, 0, 1, 192, 168]);
        check_connection_none(vec![5, 1, 0, 1], vec![5, 1, 0, 1]);
        check_connection_none(vec![5, 1, 0], vec![5, 1, 0]);

        check_connection_none(
            vec![5, 1, 0, 3, 3, 97, 97, 98, 0],
            vec![5, 1, 0, 3, 3, 97, 97, 98, 0],
        );
        check_connection_none(vec![5, 1, 0, 3, 3, 97, 97], vec![5, 1, 0, 3, 3, 97, 97]);
        check_connection_none(vec![5, 1, 0, 3, 3], vec![5, 1, 0, 3, 3]);
        check_connection_none(vec![5, 1, 0, 3], vec![5, 1, 0, 3]);
        check_connection_none(vec![5, 1, 0], vec![5, 1, 0]);
        check_connection_none(vec![], vec![]);
        // Client connection request
        // IPv4
        check_connection(
            vec![5, 1, 0, 1, 192, 168, 1, 1, 0, 80],
            VisitorRequest::Connection {
                cmd: Cmd::Connection,
                address: DstAddress::new(T::IPv4, "192.168.1.1", 80),
            },
            vec![],
        );
        // Domain
        check_connection(
            vec![5, 1, 0, 3, 3, 97, 97, 98, 0, 80],
            VisitorRequest::Connection {
                cmd: Cmd::Connection,
                address: DstAddress::new(T::Domain, "aab", 80),
            },
            vec![],
        );
    }
    fn check_none(input: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), input);
    }
    fn check_greeting(input: Vec<u8>, greeting: VisitorRequest, remain: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Greeting);
        assert_eq!(codec.proto, Proto::Socks5);
        assert_eq!(rs, Some(greeting));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_auth(input: Vec<u8>, auth: VisitorRequest, remain: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Greeting;
        codec.proto = Proto::Socks5;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Auth);
        assert_eq!(codec.proto, Proto::Socks5);
        assert_eq!(rs, Some(auth));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_auth_none(input: Vec<u8>, remain: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Greeting;
        codec.proto = Proto::Socks5;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Greeting);
        assert_eq!(codec.proto, Proto::Socks5);
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_connection(input: Vec<u8>, connection: VisitorRequest, remain: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Auth;
        codec.proto = Proto::Socks5;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Forward);
        assert_eq!(codec.proto, Proto::Socks5);
        assert_eq!(rs, Some(connection));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_connection_none(input: Vec<u8>, remain: Vec<u8>) {
        let mut codec = VisitorCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Auth;
        codec.proto = Proto::Socks5;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Auth);
        assert_eq!(codec.proto, Proto::Socks5);
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), remain);
    }
}
