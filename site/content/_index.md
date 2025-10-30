+++
title = "Prank"
sort_by = "weight"
+++

Prank is a WASM web application bundler for Rust. Prank uses a simple, optional-config pattern for building & bundling WASM, JS snippets & other assets (images, css, scss) via a source HTML file.

# Motivation

Any `wasm-bindgen`-based framework will work with Prank. If you're new to [frontend development in Rust][], [Yew][] and [Leptos][] are two popular options.

[frontend development in Rust]: https://github.com/flosse/rust-web-framework-comparison#frontend-frameworks-wasm
[Yew]: https://yew.rs/
[Leptos]: https://leptos.dev/

The easiest way to ensure that your application launches properly is to [setup your app as an executable][spago-layout] with a standard `main` function:

[spago-layout]: https://doc.rust-lang.org/spago/guide/project-layout.html

```rust
fn main() {
    // ... your app setup code here ...
}
```

Prank uses a source HTML file to drive all asset building and bundling. Prank also uses the official [dart-sass](https://github.com/sass/dart-sass), so let's get started with the following example. Copy this HTML to the root of your project's repo as `index.html`:

```html
<html>
  <head>
    <link data-prank rel="scss" href="path/to/index.scss"/>
    <link data-prank rel="rust"/>
  </head>
</html>
```

The `index.scss` file may be empty but must exist.

`prank build` will produce the following HTML at `dist/index.html`, along with the compiled scss, WASM & the JS loader for the WASM:

```html
<html>
  <head>
    <link rel="stylesheet" href="/index-fe65950190f03c21.css" integrity="sha384-pgQCpTXf5Gd2g3bMQt/1fNJvznbtkReq/e3ooBAB1MPzHOTtbFDd5/tqXjQXrP4i"/>
    
<script type="module">
import init, * as bindings from '/my_program_name-905e0077a27c1ab6.js';
const wasm = await init('/my_program_name-905e0077a27c1ab6_bg.wasm');

window.wasmBindings = bindings;
dispatchEvent(new CustomEvent("PrankApplicationStarted", {detail: {wasm}}));

</script>
  <link rel="modulepreload" href="/my_program_name-905e0077a27c1ab6.js" crossorigin="anonymous" integrity="sha384-XtIBch5nbGDblQX/VKgj2jEZMDa5+UbPgVtEQp18GY63sZAFYf81ithX9iMSLbBn"><link rel="preload" href="/my_program_name-905e0077a27c1ab6_bg.wasm" crossorigin="anonymous" integrity="sha384-Mf9hhCJLbxzecZm30W8m15djd1Z1yamaa52XBF0TsvX0/qITABYRpsB5cVmy3lt/" as="fetch" type="application/wasm"></head>
</html>
```

The contents of your `dist` dir are now ready to be served on the web.

# Installing

Please refer to the [guide](https://prankrs.dev/guide).

# Contributing

Anyone and everyone is welcome to contribute! Please review the [CONTRIBUTING.md](https://github.com/prank-rs/prank/blob/main/CONTRIBUTING.md) document for more details. The best way to get started is to find an open issue, and then start hacking on implementing it. Letting other folks know that you are working on it, and sharing progress is a great approach. Open pull requests early and often, and please use GitHub's draft pull request feature.

# License

<span><img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-blue?style=flat-square" alt="license badge"/></span>
<br>
prank is licensed under the terms of the MIT License or the Apache License 2.0, at your choosing.
