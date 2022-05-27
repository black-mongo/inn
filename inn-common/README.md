<div align="center">
   inn-common - A common library for Inn
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
- [x] Generate certificate
- [x] Parse redis-protocol for inn-ci  

## Example

```rust
use inn_common::genca::CertAuthority;
#[tokio::main]
async fn main(){
    // Certificate Authority
    CertAuthority::gen_ca("Inn Fake Ca", "Inn", "China", "Shenzhen", "ca/ca");
     let authority = CertAuthority::new(
                    "ca/ca/cacert.pem",
                    "ca/ca/cakey.pem",
                );
     // Generate Certificate with `ca/ca/cakey.pem` private key and sign with `ca/ca/cakey.pem`
     let cert = cert_authority.gen_cert_pem("www.example.com", 365);
     println!("{}", cert);
     if let Err(err) =
     std::fs::write(format!("{}/{}.cert.pem", ca.output, ca.host), cert)
     {
        eprintln!("private key file write failed: {}", err);
     }
     
}
```

## Contributing

First off, thanks for taking the time to contribute! Contributions are what makes the open-source community such an amazing place to learn, inspire, and create. Any contributions you make will benefit everybody else and are **greatly appreciated**.

Please try to create bug reports that are:

- _Reproducible._ Include steps to reproduce the problem.
- _Specific._ Include as much detail as possible: which version, what environment, etc.
- _Unique._ Do not duplicate existing opened issues.
- _Scoped to a Single Bug._ One bug per report.


## License

Inn is licensed as [MIT](./LICENSE)
