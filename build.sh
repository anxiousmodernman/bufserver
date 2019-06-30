#!/bin/bash

# Compile with musl C for statically linked binaries that can run on most Linuxes
docker run --rm -it -v "$(pwd)":/home/rust/src ekidd/rust-musl-builder cargo build --release

