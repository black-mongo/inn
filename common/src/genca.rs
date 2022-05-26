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
use moka::future::Cache;
use pem::Pem;
use rcgen::*;
use rustls::ServerConfig;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use time::ext::NumericalDuration;
use time::OffsetDateTime;
const MAX_CACHE_SIZE: u64 = 1024;
#[derive(Clone)]
pub struct CertAuthority {
    ca_key: rustls::PrivateKey,
    ca_cert: rustls::Certificate,
    cache: Cache<String, Arc<ServerConfig>>,
    serial_number: Arc<Mutex<u64>>,
}
impl CertAuthority {
    pub fn new(cert_file: String, key_file: String) -> Self {
        let private_key_bytes = fs::read(&key_file).expect("ca private key file path not valid!");
        let ca_cert_bytes = fs::read(&cert_file).expect("ca cert file path not valid!");

        let private_key = rustls_pemfile::pkcs8_private_keys(&mut private_key_bytes.as_slice())
            .expect("Failed to parse private key");

        let private_key = rustls::PrivateKey(private_key[0].clone());
        let ca_cert = rustls_pemfile::certs(&mut ca_cert_bytes.as_slice())
            .expect("Failed to parse CA certificate");
        let ca_cert = rustls::Certificate(ca_cert[0].clone());
        CertAuthority {
            ca_key: private_key,
            ca_cert,
            cache: Cache::new(MAX_CACHE_SIZE),
            serial_number: Arc::new(Mutex::new(CertAuthority::now_seconds())),
        }
    }
    pub async fn dynamic_gen_cert(&self, host: &str) -> Arc<ServerConfig> {
        if let Some(server_config) = self.cache.get(&host.to_string()) {
            return server_config;
        }

        let cert = self.gen_cert(host, 365);
        let certs: Vec<rustls::Certificate> = vec![cert];

        let server_cfg = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(certs, self.ca_key.clone())
            .expect("Failed to set certificate");
        let server_cfg = Arc::new(server_cfg);

        self.cache
            .insert(host.to_string(), Arc::clone(&server_cfg))
            .await;
        server_cfg
    }
    pub fn gen_cert_pem(&self, host: &str, days: i64) -> String {
        let cert = self.gen_cert(host, days);
        let p = Pem {
            tag: "CERTIFICATE".to_string(),
            contents: cert.0,
        };
        pem::encode(&p)
    }
    fn gen_cert(&self, host: &str, days: i64) -> rustls::Certificate {
        let mut params = rcgen::CertificateParams::default();
        {
            let serial_number = Arc::clone(&self.serial_number);
            let mut serial_number = serial_number.lock().unwrap();
            params.serial_number = Some(*serial_number);
            *serial_number += 1;
        }
        params.serial_number = Some(Self::now_seconds());
        params.not_before = OffsetDateTime::now_utc().saturating_sub(1.days());
        params.not_after = OffsetDateTime::now_utc().saturating_add(days.days());
        params
            .subject_alt_names
            .push(SanType::DnsName(host.to_string()));
        let mut distinguished_name = DistinguishedName::new();
        distinguished_name.push(DnType::CommonName, host);
        params.distinguished_name = distinguished_name;

        params.key_usages = vec![KeyUsagePurpose::DigitalSignature];
        params.extended_key_usages = vec![ExtendedKeyUsagePurpose::ServerAuth];

        let key_pair = KeyPair::from_der(&self.ca_key.0).expect("Failed to parse private key");
        params.alg = key_pair
            .compatible_algs()
            .next()
            .expect("Failed to find compatible algorithm");
        params.key_pair = Some(key_pair);

        let key_pair = KeyPair::from_der(&self.ca_key.0).expect("Failed to parse private key");

        let ca_cert_params = rcgen::CertificateParams::from_ca_cert_der(&self.ca_cert.0, key_pair)
            .expect("Failed to parse CA certificate");
        let ca_cert = rcgen::Certificate::from_params(ca_cert_params)
            .expect("Failed to generate CA certificate");

        let cert = rcgen::Certificate::from_params(params).expect("Failed to generate certificate");
        rustls::Certificate(
            cert.serialize_der_with_signer(&ca_cert)
                .expect("Failed to serialize certificate"),
        )
    }
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
    fn now_seconds() -> u64 {
        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_secs()
    }
}
