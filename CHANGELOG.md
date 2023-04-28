# v0 (Preview)

## 0.1.3
- TODO: more tests
- TODO: timing info

## 0.1.2
- Fix rust verbatim paths not supported as working directory for powershell/cmd on windows.
- Shell on windows is now resolved as `pwsh` > `powershell` > `cmd`
- Fix formatting issues with `tag` and `temp`
  - `tag` No longer stors extra newline.
  - `tag` No longer prepends whitespaces to the stored output
  - `temp` will not add a newline character if the file should be empty (i.e. only the filename is specified, no content)
- `clean` will ignore directive errors in the `.txtpp` files.

## 0.1.1
- `verify` mode now fails as soon as output differs, instead of processing the entire file and compare.
- `clean` mode cleans output of `temp` directives.
- If non-existent targets are passed as inputs, they will be an error.
- Directives in consecutive lines now work.
- Fix file not processed properly if a dependency finishes before it.

## 0.1.0
- `include`, `run`, `temp`, `tag`, `write` directives
- CLI