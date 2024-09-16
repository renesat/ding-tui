default:
    @just --list

fmt: fmt-rust fmt-nix

fmt-rust:
    cargo-fmt fmt --all

fmt-nix:
    alejandra .

clippy:
    cargo-clippy clippy

build *ARGS:
    cargo build {{ARGS}}

build-watch *ARGS:
    cargo watch -x "build -- {{ARGS}}"

run *ARGS:
    cargo run {{ARGS}}

watch *ARGS:
    cargo watch -x "run -- {{ARGS}}"
