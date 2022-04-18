
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
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

## Features

### Supported

- Socks5 protocol
- Socks5 proxy server(Noauth or UserName/Password)

### Come soon

- Http/Https proxy protocol.
- Capture packets 
- Filter packets 
- A command line management tool(inn-cli) 

## Build from source

```shell
cargo build --release 
```
The command `cargo run -p inn` start inn server on port `4556` 

## Contributing

First off, thanks for taking the time to contribute! Contributions are what makes the open-source community such an amazing place to learn, inspire, and create. Any contributions you make will benefit everybody else and are **greatly appreciated**.

Please try to create bug reports that are:

- _Reproducible._ Include steps to reproduce the problem.
- _Specific._ Include as much detail as possible: which version, what environment, etc.
- _Unique._ Do not duplicate existing opened issues.
- _Scoped to a Single Bug._ One bug per report.


## License

Inn is licensed as [MIT](./LICENSE)
