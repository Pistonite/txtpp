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
    cargo run -- clean examples -r

# Pre-commit checks
pre-commit: clean
    cargo fmt
    cargo doc

# Build and open docs
doc:
    cargo doc --open