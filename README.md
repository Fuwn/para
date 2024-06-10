# 🗃 `para`

![](assets/para.png)

[![Crate](https://img.shields.io/crates/v/para-cli.svg)](https://crates.io/crates/para-cli)
[![Build Status](https://github.com/Fuwn/para/actions/workflows/rust.yaml/badge.svg?branch=main)](https://github.com/Fuwn/para/actions/workflows/rust.yaml)

## What?

A decoder and utility for the Flipnote Studios `.ppm` animation format.

## Why this implementation over [the original](https://github.com/Flipnote-Collective/ppm-parser)?

This implementation is

- [SIGNIFICANTLY faster](#speed),
- implemented in safe Rust (a language with strict type-checking!),
- and being actively updated.

### Speed

In a 219 frame `.ppm` benchmark, running the command
`$ para ./ppms/samplememo_02.ppm gif samplememo_02.gif` on a twelve-core,
twenty-four-thread Ryzen 9 processor, para took an average of **368.35 milliseconds**,
while ppm-parser took an average of **50.4171397 seconds**.

Running the command `$ para ./ppms/samplememo_02.ppm 0 samplememo_02.png` on a twelve-core,
twenty-four-thread Ryzen 9 processor, para took an average of **1.875619 milliseconds**,
while ppm-parser took an average of **358.2232 milliseconds**.

Rust versus Python isn't very fair, however, this benchmark shows the speed improvements
that para brings to the table.

## Things To Come

- Support for other formats (namely `.kwz`)
- Optimizations here and there
- Friendlier CLI
- Possible GUI

## Getting up and Running

### Installation

#### Install from [crates.io](https://crates.io/crates/para-cli)

```shell
cargo +nightly install para-cli --force
```

#### Download from releases

Prebuilt binaries for x86_64-based Linux systems are available in the
[releases](https://github.com/Fuwn/para/releases/latest). If you are using
a different operating system or architecture such as macOS or Windows; you'll have
to build and install Para yourself!

#### Install from self-compile

```shell
cargo +nightly install --git https://github.com/Fuwn/para --branch main
```

If you are building and installing yourself, you must have
[Rust](https://www.rust-lang.org/) installed!

### Usage

```shell
usage:  para <in> <index option> <out>
index options:
        gif
        thumb
        dump
        integer(u16)
```

### Examples

- `$ para ./example.ppm 23 example.png` will output the twenty-fourth frame
  of `example.ppm` to `example.png`
- `$ para ./example.ppm thumb example.png` will output the thumbnail of
  `example.ppm` to `example.png`
- `$ para ./example.ppm dump example.json` will output the metadata of
  `example.ppm` to `example.json`
- `$ para ./example.ppm gif example.gif` will output `example.ppm`
  to `example.gif`

### Prebuilt Binaries

Prebuilt binaries for the latest release may or may not be found
[here](https://github.com/Fuwn/para/releases/latest).

## Credits

- [jaames](https://github.com/jaames) for completing PPM reverse-engineering and
  writing the [original](https://github.com/Flipnote-Collective/ppm-parser) implementation.
- [bricklife](http://ugomemo.g.hatena.ne.jp/bricklife/20090307/1236391313),
  [mirai-iro](http://mirai-iro.hatenablog.jp/entry/20090116/ugomemo_ppm),
  [harimau_tigris](http://ugomemo.g.hatena.ne.jp/harimau_tigris), and other members
  of the Japanese Flipnote community who started reverse-engineering the PPM format
  almost as soon as the app was released.
- Midmad and WDLMaster for identifying the adpcm sound codec used.
- [steven](http://www.dsibrew.org/wiki/User:Steven) and
  [yellows8](http://www.dsibrew.org/wiki/User:Yellows8) for the PPM
  documentation on DSiBrew.
- [PBSDS](https://github.com/pbsds) for more PPM reverse-engineering, as well as
  writing [hatenatools](https://github.com/pbsds/Hatenatools)

## License

[MIT License](./LICENSE)
