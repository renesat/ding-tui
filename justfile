default:
    @just --list

fmt: fmt-rust fmt-nix

fmt-rust:
    cargo-fmt fmt --all

fmt-nix:
    alejandra .

# Nix defined pre-commit
check: # .pre-commit-config.yaml
    pre-commit run -av

nix-check:
    nix flake check -v -L

clippy:
    cargo-clippy clippy

build *ARGS:
    cargo build {{ARGS}}

check-watch *ARGS:
    cargo watch -x "check -- {{ARGS}}"

build-watch *ARGS:
    cargo watch -x "build -- {{ARGS}}"

run *ARGS:
    cargo run {{ARGS}}

watch *ARGS:
    cargo watch -x "run -- {{ARGS}}"
