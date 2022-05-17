//-------------------------------------------------------------------
// MIT License
// Copyright (c) 2022 black-mongo
// @author CameronYang
// @doc
//
// @end
// Created : 2022-04-18T23:21:58+08:00
//-------------------------------------------------------------------
mod session;
use crate::session::CliSession;
use actix::clock::sleep;
use actix::{Actor, Recipient, StreamHandler};
use clap::Parser;
use common::genca::CertAuthority;
use log::{debug, error, info};
use network::codec::VisitorCodec;
use session::{CliMessage, CliResponse};
use tokio::io::split;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_util::codec::FramedRead;

/// Simple program to gen ca
#[derive(Parser)]
#[clap(name = "Inn", version, about, author)]
struct Args {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Parser)]
enum SubCommand {
    /// run proxy serve
    Run,
    /// gen your own ca cert and private key
    Genca(GenCa),
}
#[derive(Parser, Debug)]
struct GenCa {
    #[clap(
        short,
        long,
        default_value = "server",
        help = "`ca` for ca cert and `server` for server cert"
    )]
    t: String,
    #[clap(
        short,
        long,
        default_value = "ca/inn",
        help = "private key file and cert files output path"
    )]
    output: String,
    #[clap(
        short,
        long,
        default_value = "ca/ca",
        help = "ca private key file and cert files path"
    )]
    input: String,
    #[clap(long, default_value = "Inn", help = "Common Name")]
    cn: String,
    #[clap(long, default_value = "Inn", help = "Organization Name")]
    org: String,
    #[clap(short, long, default_value = "CN", help = "Country Name")]
    nation: String,
    #[clap(short, long, default_value = "CN", help = "Locality Name")]
    local: String,
    #[clap(long, default_value = "inn.com", help = "Host Name")]
    host: String,
    #[clap(
        short,
        long,
        default_value = "397",
        help = "Cert file will Expire after days"
    )]
    days: i64,
}
fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    let args = Args::parse();
    match args.subcmd {
        SubCommand::Run => run(),
        SubCommand::Genca(ca) => {
            // gen ca
            if ca.t == "ca" {
                CertAuthority::gen_ca(ca.cn, ca.org, ca.nation, ca.local, ca.output)
            } else {
                let cert_authority = CertAuthority::new(
                    format!("{}/cacert.pem", ca.input),
                    format!("{}/cakey.pem", ca.input),
                );
                let cert = cert_authority.gen_cert(ca.host.clone(), ca.days);
                debug!("{}", cert);
                if let Err(err) =
                    std::fs::write(format!("{}/{}.cert.pem", ca.output, ca.host), cert)
                {
                    error!("private key file write failed: {}", err);
                }
            }
        }
    }
}
#[actix_rt::main]
async fn run() {
    let ip = "127.0.0.1";
    let port = 4556;
    let (tx, mut rx) = mpsc::channel::<CliResponse>(10);
    let _ = actix::spawn(async move {
        if let Ok(stream) = TcpStream::connect(format!("{}:{}", ip, port)).await {
            let addr = CliSession::create(|ctx1| {
                let (r, w) = split(stream);
                CliSession::add_stream(FramedRead::new(r, VisitorCodec::default()), ctx1);
                CliSession {
                    sender: tx.clone(),
                    framed: actix::io::FramedWrite::new(w, VisitorCodec::default(), ctx1),
                }
            });
            let _ = tx.send(CliResponse::Addr(addr.recipient())).await;
        }
    })
    .await;
    let mut session: Option<Recipient<CliMessage>> = None;
    let mut n = 1;
    // receive
    loop {
        let rs = rx.try_recv();
        match rs {
            Ok(CliResponse::Addr(addr)) => {
                session = Some(addr);
            }
            Ok(CliResponse::Resp(cli)) => {
                info!("main {:?}", cli);

                if let Some(addr) = &session {
                    addr.do_send(CliMessage::Rpc(vec!["stop".into()]));
                    sleep(std::time::Duration::from_millis(200)).await;
                }
                break;
            }
            Err(tokio::sync::mpsc::error::TryRecvError::Empty) => {
                sleep(std::time::Duration::from_nanos(1)).await;
            }
            _ => {}
        }
        n += 1;
        if let Some(addr) = &session {
            if n == 2 {
                addr.do_send(CliMessage::Rpc(vec!["online_counter".into()]));
            }
        }
    }
}
