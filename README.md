tokio-bitstream-io
==================

![build](https://github.com/ramiroaisen/tokio-bitstream-io/actions/workflows/cargo-build.yml/badge.svg)
![test](https://github.com/ramiroaisen/tokio-bitstream-io/actions/workflows/cargo-test.yml/badge.svg)

## Tokio port of [bitstream-io](https://crates.io/crates/bitstream-io)

A Rust library for reading or writing binary values to or from streams
which may not be aligned at a whole byte.

This library is intended to be flexible enough to wrap
around any stream which implements the tokio `AsyncRead` or `AsyncWrite` traits.
It also supports a wide array of integer data types as
containers for those binary values.

[Documentation](https://docs.rs/tokio-bitstream-io/)
