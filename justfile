set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

default: test

tidy: clippy fmt

clippy:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo +nightly fmt --all

test:
    cargo test --workspace --all-features

tarpaulin:
    cargo tarpaulin --target-dir target-tarpaulin

doc:
    cargo +nightly doc --workspace --all-features --open
