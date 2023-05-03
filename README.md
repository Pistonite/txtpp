# txtpp
![Build Badge](https://img.shields.io/github/actions/workflow/status/Pistonite/txtpp/rust.yml)
![Version Badge](https://img.shields.io/crates/v/txtpp)
![Docs Badge](https://img.shields.io/docsrs/txtpp)
![License Badge](https://img.shields.io/github/license/Pistonite/txtpp)
![Issue Badge](https://img.shields.io/github/issues/Pistonite/txtpp)

A simple-to-use general purpose preprocessor for text files, written in rust.

You can:
- Include another file in the current file, much like C-style `#include`
- Execute a command in the middle of the current file and include the output

You can use `txtpp` both as a command line tool, or as a library with the rust crate. `txtpp` is well tested with unit tests and integration tests.

The full API doc is available on [docs.rs](https://docs.rs/txtpp)

# Installation
Install with `cargo`
```
cargo install txtpp
txtpp --help  
```

# Examples
These examples are also built with txtpp! Checkout the [docs](docs/examples) directory. More examples can be found in the [tests](tests) directory where they are used as integration tests.
## Include a file
Say you have 2 files `foo.txt.txtpp` and `bar.txt`
```
/// foo.txt.txtpp
hello
TXTPP#include bar.txt
world

/// bar.txt
bar
```

Running `txtpp foo.txt` will produce `foo.txt`:
```
/// foo.txt
hello
bar
world
```
If `bar.txt.txtpp` also exists, it will be preprocessed first to produce `bar.txt`, and the result will be used.

## Execute a command
Say you have a file `fiz.txt.txtpp`, we can run a command to include the `foo.txt.txtpp` file from the previous example:
```
/// fiz.txt.txtpp
hello                           
-TXTPP#run cat foo.txt.txtpp           
world
```
Running `txtpp fiz.txt` will produce `fiz.txt`:
```
/// fiz.txt
hello                           
hello
TXTPP#include bar.txt
world
world
```

# Table of Contents
- [Feature Summary](#feature-summary)
- [Directive Overview](#directive-overview)
  - [Syntax](#syntax)
  - [Execution](#execution)
- [Directive Specification](#directive-specification)
  - [Include Directive](#include-directive)
  - [Run Directive](#run-directive)
  - [Temp Directive](#temp-directive)
  - [Tag Directive](#tag-directive)
  - [Write Directive](#write-directive)
- [Output Specification](#output-specification)

# Feature Summary
`txtpp` provides directives that you can use in the `.txtpp` files.
A directive replaces itself with the output of the directive.
The directives are all prefixed with `TXTPP#`:
- `include` - Include the content of another file.
- `run` - Run a command and include the output of the command.
- `temp` - Store text into a temporary file next to the input file.
- `tag` - Hold the output of the next directive until a tag is seen, and replace the tag with the output.
- `write` - Write content to the output file. Can be used for escaping directives. 

# Directive Overview
## Syntax
A directive is a single- or multi-line structure in the source file, that looks like this:
```
{WHITESPACES}{PREFIX1}TXTPP#{DIRECTIVE} {ARG1}
{WHITESPACES}{PREFIX2}{ARG2}
{WHITESPACES}{PREFIX2}{ARG3}
...

```
Explanation:

1. First line:
    - `{WHITESPACES}`: Any number of whitespace characters
    - `{PREFIX1}`: Non-empty text that does not start with a whitespace character, and does not include `TXTPP#`
      - The prefix must be non-empty for multi-line directives, otherwise you won't be able to terminate it.
    - `TXTPP#`: the prefix before the directive
    - `{DIRECTIVE}`: can be one of the directives
    - ` `: (space) At least one space between the directive name and its input. This will be trimmed.
    - `{ARG1}`: argument as one string, until the end of the line. Both leading and trailing whitespaces (including the new line) will be trimmed.
1. Subsequent lines: Some directives are allowed to have more than one lines (see the [specification](#directive-specification) for what they are)
    - `{WHITESPACES}`: this must match exactly with `{WHITESPACES}` in the first line
    - `{PREFIX2}`: this must be one of:
      - the same number of space characters (i.e `' '`) as the length of `{PREFIX1}` in the first line
      - the exact same string as the `{PREFIX1}` in the first line
      - the same string as the `{PREFIX1}` in the first line without trailing whitespaces, followed by new line (i.e. arg is empty)
    - `{ARG2}`, `{ARG3}` ...: Add more arguments to the argument list. Note that `{WHITESPACES}` and `{PREFIX2}` are not included in the argument. Unlike the first line, leading whitespaces are not trimmed, but trailing whitespaces are still trimmed.
1. Ending: if a line does not match the format above, the directive ends. The ending line won't be part of this directive, but can be the start of the next directive.


For example, you can write the directive like
```
        // TXTPP#run echo "hello world"
```
which will be treated like a comment in most languages to help with syntax highlighting.

The same example as a block comment
```
        /* TXTPP#run echo "
           hello world
           "
          -TXTPP# */
```
This will execute the command `echo "hello world"`. The `-` in the last line in front of `TXTPP#` is needed to indicate that it's the start of a different directive.

## Execution
The directives are executed immediately after they are parsed. They may produce an output to be included in the output file and/or have side effects such as creating a temporary file.

If the directive has output (like `include` and `run`), it will be formated as:
- Every line in the output will be prepended with `{WHITESPACES}`, so that the indentation is consistent
    ```
       // TXTPP#run echo 1; echo 2
    ```
    Output:
    ```
       1
       2
    

    ```
- The line endings will be normalized to be [the same as the output file](#output-specification). Whether the last line has a trailing newline is persisted from the output of the command/included file. If the output from the directive doesn't have a newline character in the end, the next line from the source file will be on the same line as the last line of the directive output.
    For example:
    ```
       // TXTPP#run echo -n hello
    world

    ```
    Output:
    ```
       helloworld

    ```

Note that normally, you will not be able to connect a directive output to the previous line, since directives always start on its own line. However, you can use `tag` (see [below](#tag-directive)) to achieve this. If there is currently an active `tag` directive  that is listening for output, the output will be sent to the tag instead of the output file, without the indentation. and the directive will produce no output.

# Directive Specification
This section contains detailed specification of each directive.

## Include Directive
#### USAGE
This directive is used include the content of another file into the current file.
#### ARGUMENTS
Single-line only. The argument is `FILE_PATH`
#### BEHAVIOR
- If `FILE_PATH` is an absolute path, it will be used as is. Otherwise, it should be relative to the (directory of) the current file.
- If `FILE_PATH` does not end in `.txtpp`, and `FILE_PATH.txtpp` exists, `FILE_PATH.txtpp` will be preprocessed first to produce `FILE_PATH`, and the result will be used as the output. Note that you would still include `FILE_PATH`, not `FILE_PATH.txtpp`.
#### EXAMPLE
```
TXTPP#include foo.txt
```

## Run Directive
#### USAGE
This directive is used to run a command and include the output of the command into the current file.
#### ARGUMENTS
Can have more than one line. The arguments are joined with a single space in between to form the `COMMAND`
#### BEHAVIOR
- The `COMMAND` will be executed as a sub-process.
- Default shell selection:
  - Windows: the following is tried in order:
    1. (PowerShell 7) `pwsh -NonInteractive -NoProfile -Command COMMAND`
    2. (PowerShell 5/Windows PowerShell) `powershell -NonInteractive -NoProfile -Command COMMAND`
    3. (Command Prompt) `cmd /C COMMAND`
  - Other OS: `sh -c COMMAND`
  - You can override this with the `--shell` option.
- The working directory of the sub-process will be the directory of the current file.
- The sub-process will inherit the environment variables of the main process, with additional environment variables:
  - `TXTPP_FILE`: the path to the current file being processed. Currently this is the absolute path.
  - (that's the only environment variable for now)
#### EXAMPLE
```
TXTPP#run echo "hello world"
```
#### CAVEATS
1. `txtpp` will not run inside a `run` directive to avoid complication.
If you want to include the output of a `txtpp` file,
you can checkout [how this README file is built](docs/README.md.txtpp) for an example.


## Empty directive
#### USAGE
Empty directive has the empty string as the name and does nothing. It can be used to remove lines from the input.
#### ARGUMENTS
Can have more than one line. All arguments to the empty directive will be ignored.
#### BEHAVIOR
Nothing
#### EXAMPLE
For example, you can use it to terminate a block comment
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run ./codegen
     -arg
     -really_long_arg
     --really-really-long-option
    -TXTPP# */
}
```

If you have to put the end of the block comment in a new line, make sure to format it correctly so it is treated as part of the directive.
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run ./codegen
     -arg
     -really_long_arg
     --really-really-long-option
    -TXTPP#
    -*/
}
```

In both scenarios, the entire block comment `/**/` will be replaced with the output from running `./codegen`

## Temp Directive
#### USAGE
This directive is used to create a temporary file.
#### ARGUMENTS
Must have at least 1 argument. The first argument specifies the `FILE_PATH` to save the output (relative to the current file). The rest of the arguments are joined by [line endings](#line-endings) to form the `CONTENT`, with a trailing line ending.
#### BEHAVIOR
- `FILE_PATH` is resolved the same way as the [include directive](#include-directive)
- `FILE_PATH` cannot end in `.txtpp`. It will cause an error.
  - This is to avoid undefined behavior as the preprocessor may or may not pick up the file generated by the `temp` directive.
- `CONTENT` will be saved to `FILE_PATH`
- `FILE_PATH` will not be deleted after processing, but will be deleted with `clean`.
#### EXAMPLE
```javascript
// In this example we will export a python script,
// and use that to generate JS code from a csv
function get_cities() {
    return [
        // TXTPP#temp gen_cities.g.py
        // import csv
        //
        // with open('city.csv', 'r') as f:
        //     reader = csv.reader(f)
        //     next(reader) # skip header
        //     for city, country in reader:
        //         print(f'{{ city: "{city}", country: "{country}"}},')
        /* --- generated code --- */
        // TXTPP#run python gen_cities.g.py
        /* --- generated code --- */
    ];
}
// Note we used /*  */ to break the prefix pattern `// ` so that the directive can end
// You can also use an empty line to make it simple.
```
## Tag Directive
#### USAGE
This directive is used to create a tag to store the next directive's output.
#### ARGUMENTS
Single-line only. The argument is `TAG`
#### BEHAVIOR
- The lifecycle of a tag is as follows:
  1. A tag is created with the `tag` directive and will listen for the next directive's output.
    - If the next directive has no output (e.g. `temp`), the tag will continue to listen for the next directive's output.
  2. When there is a directive that has output, the output will be stored in the tag
  3. When there is a non-directive line that has the tag, the tag will be replaced with the stored output.
  4. After the tag is replaced, it will be deleted.
- There can only be one tag at a time to store the output of the next directive. However, multiple tags can be in the "stored" state. 
- None of the tags can be prefix of another tag. For example, if you have a tag `Foo`, you cannot have another tag `FooBar`, vice versa.
- Multiple tags can be injected into the same line of the source file. However, if the output of a tag contains another tag, it will be not replaced. The tags are replaced from left to right. If the tags overlap, the leftmost will be replaced.
- Only the first occurrence of a tag will be replaced.
- Each tag must be deleted before another tag with the same name can be created.
- The newlines in the output will be replaced by the [line endings](#line-endings) of the current file. Whether the output has a trailing newline or not will not be changed.
  - Example: if the stored output has no trailing newline, the part after the tag will be on the same line as the last line in the stored output.
- If there are any unused tags in the end of the file, there will be an error.


#### EXAMPLE
In this example, we want the output to be exactly as is because of the `<pre>` tag. The output of the `run` directive will be put in the `<pre>` tag.
```html
<div>
  <!-- TXTPP#tag PRE_CONTENT -->
  <!-- TXTPP#run python gen_pre.py

       TXTPP# -->
  <pre>PRE_CONTENT --></pre>
</div>
```
The following is invalid because the tag is used before the output is stored.
```html
<div>
  <!-- TXTPP#tag PRE_CONTENT -->
  <pre>PRE_CONTENT --></pre>
  <!-- TXTPP#run python gen_pre.py

       TXTPP# -->
</div>
```

## Write Directive
#### USAGE
This directive writes its arguments to the output file. It can be used to escape other directives.
#### ARGUMENTS
Can have more than one line. Each argument is one line in the output
#### BEHAVIOR
- The arguments will be written to the output file as is.
- Because the content is treated as output of a directive, tags won't be injected.
#### EXAMPLE
```
-TXTPP#write the line below will be written to the output file as is
-TXTPP#run echo "hello world"
stuff

```
Output
```
the line below will be written to the output file as is
TXTPP#run echo "hello world"stuff

```
(To put `stuff` on its own line, add an extra line to the `write` directive)
# Output Specification
This section specifies details of the output of the preprocessor.
## Line endings
The output files and temporary output files will have consistent line ending with the input `.txtpp` files. If the input file has mixed line endings, the output file will have the same line endings as the first line in the input file.

If the input file does not have a line ending, the output file will have the same line ending as the operating system (i.e. `\r\n` on Windows, `\n` on Unix).

The output files will have a trailing newline unless `--no-trailing-newline` is specified. The flag will not affect the temporary output files, however. Whether a temporary file has a trailing newline depends on if the directive has an empty line in the end.
