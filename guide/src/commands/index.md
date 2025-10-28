# Commands

Prank ships with a set of CLI commands to help you in your development workflows.

## build

`prank build` runs a spago build targeting the wasm32 instruction set, runs `wasm-bindgen` on the built WASM, and spawns
asset build pipelines for any assets defined in the target `index.html`.

Prank leverages Rust's powerful concurrency primitives for maximum build speeds & throughput.

## watch

`prank watch` does the same thing as `prank build`, but also watches the filesystem for changes, triggering new builds
as changes are detected.

## serve

`prank serve` does the same thing as `prank watch`, but also spawns a web server.

## clean

`prank clean` cleans up any build artifacts generated from earlier builds.

## config show

`prank config show` prints out Prank's current config, before factoring in CLI arguments. Nice for testing & debugging.

## tools show

`prank tools show` prints out information about tools required by prank and the project. It shows which tools are
expected and which are found. 
