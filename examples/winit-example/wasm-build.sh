#!/bin/sh
cargo build --target wasm32-unknown-unknown 
wasm-bindgen --out-dir dist --web target/wasm32-unknown-unknown/debug/wasm-example.wasm