# nes-rs
[![GithubCi](https://github.com/selassje/nes-rs/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/selassje/nes-rs/actions/workflows/ci.yml)

# about

This is my NES emulator implementation written in Rust. My main goal of this project is to practice the language :)

You can check out the web version at:
https://selassje.github.io/nes-rs/

# building

The CI tested targets are **x86_64-pc-windows-msvc** and **x86_64-unknown-linux-gnu** 

Before running cargo build for those targets you will need to build the SDL2 lib via the help of [cargo-vcpkg](https://github.com/mcgoo/cargo-vcpkg)

* `cargo install cargo-vcpkg`
* `cargo vcpkg build`
* `cargo build --release`


Target **wasm32-unknown-emscripten** is also supported.
In that case, instead of building SDL2, [Emscripten SDK](https://emscripten.org/docs/getting_started/downloads.html) 
is required which provides SDL2 as well as OpenGL ports.
The Emscripten SDK version known to work with this project is **2.0.9**.

I recommend using [cargo-web](https://github.com/koute/cargo-web) to build and run the emscripten target

* `cargo web start --release`


