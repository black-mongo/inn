<div align="center">
  inn-network - A network library for Inn

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

## Example
```rust
use actix::System;
use inn_network::{proxy::Proxy, NetWork};
#[actix_rt::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    //
    let sock5 = async {
        let _ = NetWork.start("127.0.0.1", 4556, || {}).await;
    };
    let http_proxy = async {
        Proxy::start_proxy("127.0.0.1:4557", "ca/ca/cacert.pem", "ca/ca/cakey.pem").await;
    };
    tokio::join!(sock5, http_proxy);
    System::current().stop();
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
