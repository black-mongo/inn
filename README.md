<div align="center">
  Inn - A cross-platform packet capture tool similar to fiddler and whistle

  <br />
  <br />
  <a href="https://github.com/black-mongo/inn/issues/new?assignees=&labels=&template=bug_report.md&title=">Report a Bug</a>
  ·
  <a href="https://github.com/black-mongo/inn/issues/new?assignees=&labels=&template=feature_request.md&title=">Request a Feature</a>
  .
  <a href="https://github.com/black-mongo/inn/issues/new?assignees=&labels=&template=feature_request.md&title=">Ask a Question</a>
<br />
<br />
</div>


[![Build and Test](https://github.com/black-mongo/inn/workflows/Build%20and%20Test/badge.svg)](https://github.com/black-mongo/inn/actions?query=workflows%3A%22Build+and+Test%22)
[![codecov](https://codecov.io/gh/black-mongo/inn/branch/main/graph/badge.svg?token=J562YL59IB)](https://codecov.io/gh/black-mongo/inn)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Features

### Supported

- [x] Socks5 protocol
- [x] Socks5 proxy server(Noauth or UserName/Password)
- [x] Http/Https proxy protocol.
- [x] Command line tool(inn-cli) to sign server certificate
- [x] Act as your own certificate authority (CA) using the command line tool(inn-cli)
- Capture packets 
- Filter packets 
- A command line management tool(inn-cli) 
- Js virtual machine

## Getting Started 
### Step 1, Clone the source 
```shell
$ git clone https://github.com/black-mongo/inn.git
$ cd inn
```
### Step 2, Generate private key file and certificate file for ca
```shell
$ cargo run -p inn_cli -- genca -t ca -o ca/ca
    Finished dev [unoptimized + debuginfo] target(s) in 0.68s
     Running `target/debug/inn_cli genca -t ca -o ca/ca`
[2022-05-26T17:05:09Z DEBUG common::genca] ca/ca/cacert.pem
    -----BEGIN CERTIFICATE-----
    MIIBpzCCAU2gAwIBAgIJAO2/sI9ZEVIeMAoGCCqGSM49BAMCMDYxDDAKBgNVBAMM
    A0lubjEMMAoGA1UECgwDSW5uMQswCQYDVQQGDAJDTjELMAkGA1UEBwwCQ04wIBcN
    NzUwMTAxMDAwMDAwWhgPNDA5NjAxMDEwMDAwMDBaMDYxDDAKBgNVBAMMA0lubjEM
    MAoGA1UECgwDSW5uMQswCQYDVQQGDAJDTjELMAkGA1UEBwwCQ04wWTATBgcqhkjO
    PQIBBggqhkjOPQMBBwNCAAQgy7m1BWukJr8VAOxie56u+vBcaEOCZtu+k7x155B8
    +gq+JuKSZoMRJwEmIjbgnPErMR6qzEYEXjH56Oiu09Ago0IwQDAOBgNVHQ8BAf8E
    BAMCAYYwHQYDVR0OBBYEFB5SEVmPsL/tB9jDPvn9xvLs6qJuMA8GA1UdEwEB/wQF
    MAMBAf8wCgYIKoZIzj0EAwIDSAAwRQIhAI8XyypwZC9Uzl457YGTiSK54kjtrrsO
    hzY7kEjlTNioAiA0rn8LmSPPzn3aXJ/CPxTYvuSUx2V4O98Zn7cs2BVtNA==
    -----END CERTIFICATE-----
    
[2022-05-26T17:05:09Z DEBUG common::genca] ca/ca/cakey.pem
    -----BEGIN PRIVATE KEY-----
    MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgf2kKonKFODJuoQDv
    qN3CohJ1jhIUptc1UuQja26ieJWhRANCAAQgy7m1BWukJr8VAOxie56u+vBcaEOC
    Ztu+k7x155B8+gq+JuKSZoMRJwEmIjbgnPErMR6qzEYEXjH56Oiu09Ag
    -----END PRIVATE KEY-----
``` 
The command `cargo run -p inn` start inn sock5 server on port `4556` and https_proxy server on port `4557`

### Step3, Add `ca/ca/cacert.pem` to your system root certificate storage and trust it

### Step4, Setting https proxy to `http://127.0.0.1:4557` with your browser

### Step5, Open your browser with `https://www.google.com`
So far only www.google.com with MITM and other https web site will through tunnel, future add rules with MITM or tunnel

## Contributing

First off, thanks for taking the time to contribute! Contributions are what makes the open-source community such an amazing place to learn, inspire, and create. Any contributions you make will benefit everybody else and are **greatly appreciated**.

Please try to create bug reports that are:

- _Reproducible._ Include steps to reproduce the problem.
- _Specific._ Include as much detail as possible: which version, what environment, etc.
- _Unique._ Do not duplicate existing opened issues.
- _Scoped to a Single Bug._ One bug per report.


## License

Inn is licensed as [MIT](./LICENSE)
