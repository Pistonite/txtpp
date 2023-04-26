install:
    rustup update
    echo "cargo fmt" > .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

todo:
    grep -rn TODO src examples tests CHANGELOG.md README.md