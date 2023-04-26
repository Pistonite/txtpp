# v0 (Preview)

## 0.2.1
- TODO: more docs
- TODO: more tests

## 0.2.0
- `verify` mode now fails as soon as output differs, instead of processing the entire file and compare.
- `clean` mode cleans output of `temp` directives.
- If non-existent targets are passed as inputs, they will be an error.
- Directives in consecutive lines now work.
- Fix file not processed properly if a dependency finishes before it.

## 0.1.0
- `include`, `run`, `temp`, `tag`, `write` directives
- CLI