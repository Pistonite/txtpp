# CHANGELOG

## 0.2.4
- Fixed bug where `run` directives still executed when dependency is not built yet

## 0.2.2
- New `after` directive to specify dependency explicitly

## 0.2.1
- Temp files are no longer re-written if they are already up-to-date in both verify and build mode 
- New flag `--needed/-N` and corresponding mode `InMemoryBuild` that stores the fresh output in memory and only writes the file if different

## 0.2.0
- Verify mode no longer re-writes temp files if they are already up-to-date.

## 0.1.5
- Migrate repo and update links. (otherwise same as 0.1.4)

## 0.1.4
- Fixed a bug where directives are not displayed correctly in error messages.
- `temp` directives will now error if the export path is a `.txtpp` file.
- Improved console output and error messages
- `txtpp` binary will now error if ran as a subcommand (i.e. if `TXTPP_FILE` environment variable is set)
- Fixed a bug with the `write` directive where the first argument is skipped in the output.

## 0.1.3 (Preview)
- More consistent handling of trailing newlines:
  - Directive output will be written to the output as-is. If the output has no trailing newline (for example, the included file has no trailing newline, or if the run output prints no newline in the end), the next line in the source will be on the same line as the last line of the directive output.
  - Temporary files will have a trailing newline if the `temp` directive has an empty line in the end, and vice versa.
  - Output files will always have a trailing newline, or always have no trailing newlines with the `--no-trailing-newline` flag.
- Unused tags at the end of the file will now be an error.

## 0.1.2 (Preview)
- Fix rust verbatim paths not supported as working directory for powershell/cmd on windows.
- Shell on windows is now resolved as `pwsh` > `powershell` > `cmd`
- Fix formatting issues with `tag` and `temp`
  - `tag` No longer stores extra newline.
  - `tag` No longer prepends whitespaces to the stored output
  - `temp` will not add a newline character if the file should be empty (i.e. only the filename is specified, no content)
- `clean` will ignore directive errors in the `.txtpp` files.

## 0.1.1 (Preview)
- `verify` mode now fails as soon as output differs, instead of processing the entire file and compare.
- `clean` mode cleans output of `temp` directives.
- If non-existent targets are passed as inputs, they will be an error.
- Directives in consecutive lines now work.
- Fix file not processed properly if a dependency finishes before it.

## 0.1.0 (Preview)
- `include`, `run`, `temp`, `tag`, `write` directives
- CLI
