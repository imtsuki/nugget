# Nugget

> Who hates nuggets?

A WebGPU renderer written in Rust, using the `wgpu` crate. It is capable of loading glTF files and can run both natively and in the browser via WebAssembly.

![Screenshot](screenshot.png)

_Credit: [GAMEBOY by grimmorf](https://sketchfab.com/3d-models/gameboy-4a1da0cefa904c4eae895338bd6f3334)_

## Getting Started

To run natively, run the following command:

```bash
cargo run --release <PATH>
```

where `<PATH>` is the path to a glTF file.

To run Nugget in the browser, you will need to install `wasm-pack` first. Then run the following command to build the project:

```bash
wasm-pack build --target web
```

This will generate a `pkg` directory containing the compiled WebAssembly module. Then, serve the files on `localhost:8000` using a command like:

```bash
python -m http.server 8000
```

Please note that currently, Chromium browsers are the only supported browser for WebGPU (Firefox's WebIDL is out of date, see [Bug 1785576](https://bugzilla.mozilla.org/show_bug.cgi?id=1785576)). Also, the origin trial token included in `index.html` that enables WebGPU is for `localhost:8000` only. If you wish to run on a different port, you need to manually enable the corresponding features in `chrome://flags`, or generate your own token.

## License

nugget is licensed under the [MIT License](LICENSE).
