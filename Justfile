set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Install/update dependencies
install:
    rustup update
    cargo install cargo-watch

# List TODOs
todo:
    grep -rn TODO src examples tests CHANGELOG.md README.md

# Clean example output
clean:
    cargo run -- clean tests/examples docs -r

# Generate the readme
readme:
    cargo run docs/README.md
    mv docs/README.md README.md

# Pre-commit checks
pre-commit: && readme clean
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt
    cargo doc
    cargo test

# Build and open docs
doc: pre-commit
    cargo doc --open