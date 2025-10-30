# Prank

[![Build Status](https://github.com/prank-rs/prank/actions/workflows/ci.yaml/badge.svg)](https://github.com/prank-rs/prank/actions)
[![](https://img.shields.io/crates/v/prank.svg?color=brightgreen&style=flat-square)](https://crates.io/crates/prank)
![](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square)
[![Discord Chat](https://img.shields.io/discord/793890238267260958?logo=discord&style=flat-square)](https://discord.gg/JEPdBujTDr)
[![](https://img.shields.io/crates/d/prank?label=downloads%20%28crates.io%29&style=flat-square)](https://crates.io/crates/prank)
[![](https://img.shields.io/github/downloads/prank-rs/prank/total?label=downloads%20%28GH%29&style=flat-square)](https://github.com/prank-rs/prank/releases)
![](https://img.shields.io/homebrew/installs/dy/prank?color=brightgreen&label=downloads%20%28brew%29&style=flat-square)

**Build, bundle & ship your Rust WASM application to the web.**
<br/>
*”Pack your things, we’re going on an adventure!” ~ Ferris*

Prank is a WASM web application bundler for Rust. Prank uses a simple, optional-config pattern for building & bundling WASM, JS snippets & other assets (images, css, scss) via a source HTML file.

**📦 Dev server** - Prank ships with a built-in server for rapid development workflows, as well as support for HTTP & WebSocket proxies.

**🏗 Change detection** - Prank watches your application for changes and triggers builds for you, including automatic browser reloading.

## Getting Started

Head on over to the [Prank website](https://prankrs.dev), everything you need is there. A few quick links:

- [Install](https://prankrs.dev/#install)
  - Download a released binary: https://github.com/prank-rs/prank/releases
  - `spago binstall prank` (installing a pre-compiled binary using [spago-binstall](https://github.com/spago-bins/spago-binstall))
  - `spago install prank --locked` (compile your own binary from crates.io)
  - `spago install --git https://github.com/prank-rs/prank prank` (compile your own binary from the most recent git commit)
  - `spago install --path . prank` (compile your own binary from your local source)
  - `brew install prank` (installing from [Homebrew](https://brew.sh/))
  - `nix-shell -p prank` (installing from [nix packages](https://nixos.org/))
- [App Setup](https://prankrs.dev//#app-setup)
- [Assets](https://prankrs.dev/assets/)
- [Configuration](https://prankrs.dev/configuration/)
- [CLI Commands](https://prankrs.dev/commands/)

## Examples

Check out the example web applications we maintain in-repo under the `examples` directory.

## Contributing

Anyone and everyone is welcome to contribute! Please review the [CONTRIBUTING.md](./CONTRIBUTING.md) document for more details. The best way to get started is to find an open issue, and then start hacking on implementing it. Letting other folks know that you are working on it, and sharing progress is a great approach. Open pull requests early and often, and please use GitHub's draft pull request feature.

### License

prank is licensed under the terms of the MIT License or the Apache License 2.0, at your choosing.
