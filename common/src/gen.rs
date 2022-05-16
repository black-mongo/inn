//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-05-17T00:28:36+08:00
//-------------------------------------------------------------------
extern crate rcgen;
use log::debug;
use log::error;
use rcgen::*;
use std::fs;
pub fn gen_ca(
    common_name: String,
    oranization_name: String,
    country_name: String,
    locality_name: String,
    out: String,
) {
    let mut params = CertificateParams::default();
    let mut distinguished_name = DistinguishedName::new();
    distinguished_name.push(DnType::CommonName, common_name);
    distinguished_name.push(DnType::OrganizationName, oranization_name);
    distinguished_name.push(DnType::CountryName, country_name);
    distinguished_name.push(DnType::LocalityName, locality_name);
    params.distinguished_name = distinguished_name;
    params.key_usages = vec![
        KeyUsagePurpose::DigitalSignature,
        KeyUsagePurpose::KeyCertSign,
        KeyUsagePurpose::CrlSign,
    ];
    params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    let cert = Certificate::from_params(params).unwrap();
    let cert_crt = cert.serialize_pem().unwrap();
    let _ = fs::create_dir(&out);
    let cert_file = format!("{}/cert.pem", &out);
    let key_file = format!("{}/key.pem", &out);
    debug!("{}\n{}", cert_file, cert_crt);
    if let Err(err) = fs::write(cert_file, cert_crt) {
        error!("cert file write failed: {}", err);
    }

    let private_key = cert.serialize_private_key_pem();
    debug!("{}\n{}", key_file, private_key);
    if let Err(err) = fs::write(key_file, private_key) {
        error!("private key file write failed: {}", err);
    }
}
