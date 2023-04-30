# v0 (Preview)

## 0.1.4
- TODO: timing info
- TODO: more tests
- TODO: detect TXTPP as subcommand
- TODO: temp cannot save .txtpp

## 0.1.3
- More consistent handling of trailing newlines:
  - Directive output will be written to the output as-is. If the output has no trailing newline (for example, the included file has no trailing newline, or if the run output prints no newline in the end), the next line in the source will be on the same line as the last line of the directive output.
  - Temporary files will have a trailing newline if the `temp` directive has an empty line in the end, and vice versa.
  - Output files will always have a trailing newline, or always have no trailing newlines with the `--no-trailing-newline` flag.
- Unused tags at the end of the file will now be an error.

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