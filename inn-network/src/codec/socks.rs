use byteorder::BigEndian;
use byteorder::ByteOrder;
use bytes::BytesMut;

use crate::server::ProxyServer;

use super::AuthChoice;
use super::Cmd;
use super::DstAddress;
use super::T;
//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-19T09:49:23+08:00
//-------------------------------------------------------------------
use super::VisitorDecoder;
use super::VisitorRequest;

use super::Proto;
use super::State;
use std::io::{Error, ErrorKind};
pub(crate) struct SocksCodec {
    pub state: State,
}
impl Default for SocksCodec {
    fn default() -> Self {
        SocksCodec {
            state: State::Undefined,
        }
    }
}

impl VisitorDecoder for SocksCodec {
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<VisitorRequest>, Error> {
        if src.len() < 3 {
            return Ok(None);
        }
        match &self.state {
            State::Undefined => {
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
                        Ok(Some(VisitorRequest::Greeting {
                            proto: Proto::Socks5,
                            auth: buf.to_vec(),
                        }))
                    }
                } else {
                    let msg: String = "Invalid socks5 protocol".into();
                    Err(Error::new(ErrorKind::Other, msg))
                }
            }
            State::Greeting => {
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
            State::Auth => {
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
            State::Forward => Ok(Some(VisitorRequest::Forward(
                src.split_to(src.len()).to_vec(),
            ))),
        }
    }
}
