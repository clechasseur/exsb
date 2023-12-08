set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

toolchain := ''
trimmed_toolchain := trim(toolchain)

cargo := if trimmed_toolchain != "" {
    "cargo +" + trimmed_toolchain
} else {
    "cargo"
}

open := '--open'

default:
    @just --list

tidy: clippy fmt

clippy:
    {{cargo}} clippy --workspace --all-targets --all-features -- -D warnings

fmt:
    cargo +nightly fmt --all

check:
    {{cargo}} check --workspace --all-targets --all-features

build *extra_args:
    {{cargo}} build --workspace --all-targets --all-features {{extra_args}}

run *extra_args:
    {{cargo}} run --all-features {{extra_args}}

test *extra_args:
    {{cargo}} test --workspace --all-features {{extra_args}}

tarpaulin:
    {{cargo}} tarpaulin --target-dir target-tarpaulin

pre-msrv:
    mv Cargo.toml Cargo.toml.bak
    mv Cargo.lock Cargo.lock.bak
    mv Cargo.toml.msrv Cargo.toml
    mv Cargo.lock.msrv Cargo.lock

post-msrv:
    mv Cargo.toml Cargo.toml.msrv
    mv Cargo.lock Cargo.lock.msrv
    mv Cargo.toml.bak Cargo.toml
    mv Cargo.lock.bak Cargo.lock

msrv:
    {{ if path_exists("Cargo.lock.msrv") == "true" { `just pre-msrv` } else { ` ` } }}
    cargo msrv -- cargo check --workspace --lib --bins --all-features
    {{ if path_exists("Cargo.lock.bak") == "true" { `just post-msrv` } else { ` ` } }}

doc $RUSTDOCFLAGS="-D warnings":
    {{cargo}} doc --workspace --all-features {{open}}

doc-coverage $RUSTDOCFLAGS="-Z unstable-options --show-coverage":
    cargo +nightly doc --no-deps --workspace --all-features

test-package:
    {{cargo}} publish --dry-run
