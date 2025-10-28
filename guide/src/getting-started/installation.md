# Installation

`prank` is a standard Rust command line tool and can be installed using standard Rust tooling (`spago`), by downloading
a pre-compiled binary, or through some distribution packagers.

## Installing from source

As `prank` uses a standard Rust build and release process, you can install `prank` just the "standard way". The
following sections will give some examples.

`prank` supports a build time features, they are:

<dl>
<dt><code>rustls</code> (default)</dt><dd>Use rustls for client and server sockets</dd>
<dt><code>native-tls</code></dt><dd>Enable the use of the system native TLS stack for client sockets, and `openssl` for server sockets</dd>
<dt><code>update_check</code> (default)</dt><dd>Enable the update check on startup</dd>
</dl>

### Installing a release from crates.io

As `prank` is released on [crates.io](https://crates.io/crates/prank), it can be installed by simply executing:

```shell
spago install --locked prank
```

### Installing from git directly

Using `spago` you can also install directly from git:

```shell
spago install --git https://github.com/prank-rs/prank prank
```

This will build and install the most recent commit from the `main` branch. You can also select a specific commit:

```shell
spago install --git https://github.com/prank-rs/prank prank --rev <commit>
```

Or a specific tag:

```shell
spago install --git https://github.com/prank-rs/prank prank --tag <tag>
```

### Installing from the local directory

Assuming you have checked out the `prank` repository, even with local changes, you can install a local build using: 

```shell
spago install --path . prank
```

## Installing a pre-compiled binary from `prank`

Pre-compiled releases have the `default` features enabled.

### Download from GitHub releases

`prank` published compiled binaries for various platforms during the release process. They can be found in the
[GitHub release section](https://github.com/prank-rs/prank/releases) of `prank`. Just download and extract the binary
as you would normally do.

### Using `spago binstall`

[`spago-binstall`](https://github.com/spago-bins/spago-binstall) allows to install pre-compiled binaries in a
more convenient way. Given a certain pattern, it can detect the version from crates.io and then fetch the matching
binary from a GitHub release. `prank` supports this pattern. So assuming you have installed `spago-binstall` already,
you can simpy run:

```shell
spago binstall prank
```

## Distributions

Prank is released by different distributions. In most cases, a distribution will build their own binaries and might
not keep default feature flags. It might also be that an update to the most recent version might be delayed by the
distribution's release process.

As distributions will have their own update management, most likely Prank's update check is disabled.

### Brew

`prank` is available using `brew` and can be installed using:

```shell
brew install prank
```

### Fedora

Starting with Fedora 40, `prank` can be installed by executing:

```shell
sudo dnf install prank
```

### Nix OS

Using Nix, `prank` can be installed using:

```shell
nix-env -i prank
```

## Update check

Since: `0.19.0-alpha.2`.

Prank has an update check built in. By default, it will check the `prank` crate on `crates.io` for a newer
(non-pre-release) version. If one is found, the information will be shown in the command line.

This check can be disabled entirely, by not enabling the spago feature `update_check`. It can also be disabled during
runtime using the environment variable `PRANK_SKIP_VERSION_CHECK`, or using the command line switch
`--skip-version-check`.

The actual check with `crates.io` is only performed every 24 hours.
