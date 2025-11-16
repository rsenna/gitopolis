# Vaquera Development Guide

## Table of Contents

TODO

## High Level Overview

The user visible interface and high level integration is in the root crate
(see `./src`).

### Key Directories

- `crates/` - Core runtime crates
- `src/` - User-facing CLI implementation, subcommands
- `tests/spec/` - Integration tests (spec tests)
- `tests/unit/` - Extra unit tests
- `tests/data/` - Test fixtures and data files

## Quick Start

### Building Vaquera

To compile after making changes:

```bash
cargo build
```

For faster iteration during development (less optimisation):

```bash
cargo build --bin vaq
```

Execute your development build:

```bash
./target/debug/vaq
```

### Running with your changes

```bash
# TODO
# Run a local file
# ./target/debug/vaq

# Run with permissions

# Run the REPL
```

## Commands

### Compilation and Checks

```bash
# Check for compilation errors (fast, no binary output)
cargo check

# Check specific package
cargo check -p vaq_core

# Build release version (slow, optimised)
cargo build --release
```

### Code Quality

```bash
# Run linter (only show suggestions)
cargo clippy

# Format the code (apply all suggestions)
cargo clippy --fix
```

## Testing

### Running Tests

```bash
# Run all tests (this takes a while)
cargo test

# Filter tests by name
cargo test <nameOfTest>

# Run tests in a specific package
cargo test -p vaq_core

# Run spec tests only
cargo test specs

# Run a specific test
cargo test unit::test_name
```

### Test Organisation

- **Spec tests** (`tests/specs/`) - Main E2E tests
- **Unit tests** - Inline with source code in each module
- **Additional tests** (`/tests/`) - Additional unit or integration tests

## "spec" tests

The main form of E2E test in Vaquera is the "spec" test. These tests can be
found in `tests/specs/`.

### Creating a New Spec Test

- Create a file in `tests/specs/<test-name>.rs` using a descriptive name
- Add any input (`.in`) or output (`.out`) files you need for the new test at
   `tests/data`

## Development Workflows

### Updating Dependencies

```bash
# Update Cargo dependencies
cargo update

# Update to latest compatible versions
cargo upgrade  # Requires cargo-edit: cargo install cargo-edit

# Check for outdated dependencies
cargo outdated  # Requires cargo-outdated
```

## Debugging

### Debugging Rust Code

Use your IDE's debugger (VS Code with rust-analyzer, IntelliJ IDEA, etc.):

1. Set breakpoints in Rust code
2. Run tests in debug mode through your IDE
