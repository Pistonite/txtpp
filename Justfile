set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

# Install/update dependencies
install:
    rustup update
    cargo install cargo-watch

# Clean example output
clean:
    cargo run --features cli -- clean tests/examples docs -r

# Generate the readme
readme:
    cargo run --features cli docs/README.md
    mv docs/README.md README.md

# Pre-commit checks
check: && readme clean
    cargo clippy --all-targets --all-features -- -D warnings
    cargo fmt
    cargo doc
    cargo test

# Build and open docs
doc: check
    cargo doc --open
