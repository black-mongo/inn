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
use std::io::Error;
use std::io::ErrorKind;

use crate::codec::cli::CliCodec;
use crate::codec::socks::SocksCodec;
mod cli;
pub mod forward;
pub mod socks;
trait VisitorDecoder {
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VisitorRequest>, Error>;
}
pub struct VisitorCodec {
    proto: Proto,
    handler: Option<Box<dyn VisitorDecoder>>,
}
impl Default for VisitorCodec {
    fn default() -> Self {
        VisitorCodec {
            proto: Proto::Undefined,
            handler: None,
        }
    }
}
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone)]
pub(crate) enum State {
    Undefined,
    Greeting,
    Auth,
    Forward,
}
#[derive(Debug, PartialEq, Clone)]
pub enum Proto {
    Undefined,
    Socks5,
    Cli,
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
    Cli(common::cli::Cli),
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
    Cli(common::cli::Cli),
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
            VisitorResponse::Cli(cli) => cli.into(),
        }
    }
}
impl Decoder for VisitorCodec {
    type Item = VisitorRequest;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        trace!("Client data:{:?}", src.to_vec());

        // check_proto
        if self.proto == Proto::Undefined {
            if src.as_ref()[0] == 0x05 {
                self.proto = Proto::Socks5;
                self.handler = Some(Box::new(SocksCodec::default()))
            } else if src.as_ref()[0] == b'*' {
                self.proto = Proto::Cli;
                self.handler = Some(Box::new(CliCodec::default()))
            }
        }
        match &mut self.handler {
            Some(handler) => handler.decode(src),
            _ => Err(Error::new(ErrorKind::Other, "Invalid protocol")),
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
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), input);
    }
    fn check_greeting(input: Vec<u8>, greeting: VisitorRequest, remain: Vec<u8>) {
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Greeting);
        assert_eq!(rs, Some(greeting));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_auth(input: Vec<u8>, auth: VisitorRequest, remain: Vec<u8>) {
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Greeting;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Auth);
        assert_eq!(rs, Some(auth));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_auth_none(input: Vec<u8>, remain: Vec<u8>) {
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Greeting;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Greeting);
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_connection(input: Vec<u8>, connection: VisitorRequest, remain: Vec<u8>) {
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Auth;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Forward);
        assert_eq!(rs, Some(connection));
        assert_eq!(bytes.to_vec(), remain);
    }
    fn check_connection_none(input: Vec<u8>, remain: Vec<u8>) {
        let mut codec = SocksCodec::default();
        let mut bytes = BytesMut::from(input.as_slice());
        codec.state = State::Auth;
        let rs = codec.decode(&mut bytes).unwrap();
        assert_eq!(codec.state, State::Auth);
        assert_eq!(rs, None);
        assert_eq!(bytes.to_vec(), remain);
    }
}
