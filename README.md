# cargo-bump

[![crates.io](https://img.shields.io/crates/v/cargo-bump.svg)](https://crates.io/crates/cargo-bump)
[![build status](https://travis-ci.org/wraithan/cargo-bump.svg?branch=master)](https://travis-ci.org/wraithan/cargo-bump)

This adds the command `cargo bump` which bumps the current version in your
`Cargo.toml`.

This is meant to be a clone of `npm version` with the `pre*` version specifiers
omitted as I rarely see the pre-release versions on [crates.io](https://crates.io/).

## installation

Install using cargo:

`cargo install cargo-bump`

## examples

Increment the patch version: `cargo bump` or `cargo bump patch`

Increment the minor version and create a git tag: `cargo bump minor --git-tag`

Set the version number directly: `cargo bump 13.3.7`

## usage

```text
USAGE: cargo bump <SEMVER | major | minor | patch> [FLAGS]

    Version: ${PREFIX}${MAJOR}.${MINOR}.${PATCH}-${PRE-RELEASE}+${BUILD}
    Example: v3.1.4-alpha+159

ARGS:
    <SEMVER>    Must be 'major', 'minor', 'patch' or a semantic version string:
                https://semver.org

OPTIONS:
    -b, --build <BUILD>                Add build part to version, e.g. 'dirty'
    -g, --git-tag                      Commit the updated version and create a git tag
    -h, --help                         Print help information
        --ignore-lockfile              Don't update Cargo.lock
        --manitest-path <PATH>         Path to Cargo.toml
    -p, --pre-release <PRE-RELEASE>    Add pre-release part to version, e.g. 'beta'
    -r, --run-build                    Require `cargo build` to succeed (and update Cargo.lock)
                                       before running git actions
    -t, --tag-prefix <PREFIX>          Prefix to the git-tag, e.g. 'v' (implies --git-tag)
    -V, --version                      Print version information
```
