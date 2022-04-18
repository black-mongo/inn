//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-17T20:39:01+08:00
//-------------------------------------------------------------------

pub struct ForwardCodec;

use crate::codec::*;
use actix_codec::Decoder;
use actix_codec::Encoder;

impl Encoder<Vec<u8>> for ForwardCodec {
    type Error = Error;
    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend(item);
        Ok(())
    }
}
impl Decoder for ForwardCodec {
    type Item = Vec<u8>;
    type Error = Error;
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.is_empty() {
            Ok(None)
        } else {
            Ok(Some(src.split_to(src.len()).to_vec()))
        }
    }
}
