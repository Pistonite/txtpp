install:
    rustup update
    echo "#!/bin/sh" > .git/hooks/pre-commit
    echo "echo running cargo fmt" >> .git/hooks/pre-commit
    echo "cargo fmt" >> .git/hooks/pre-commit
    echo "git add ." >> .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit

todo:
    grep -rn TODO src examples tests CHANGELOG.md README.md