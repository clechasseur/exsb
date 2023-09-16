# Rust project template

This is a simple template repository for Rust projects that includes some default workflows.

## Usage

1. Create a new repository using this template
2. Clone your new repository
3. Run `cargo new` to create a Rust project at the repository root<br/>
   OR<br/>
   Run `cargo new` from the repository root to create a new Rust project, then create a root `Cargo.toml` to setup a [workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
4. Adjust workflows if required
   * In particular, in order to have code coverage work, setup your project on [codecov.io](https://about.codecov.io/) and create a `CODECOV_TOKEN` secret for your repository's actions
5. Add status badges to your readme if you feel like it
6. ???
7. Profit!
