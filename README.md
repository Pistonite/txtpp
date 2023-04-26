# txtpp
![Build Badge](https://img.shields.io/github/actions/workflow/status/iTNTPiston/txtpp/rust)
![Version Badge](https://img.shields.io/crates/v/txtpp)
![Docs Badge](https://img.shields.io/docsrs/txtpp)
![License Badge](https://img.shields.io/github/license/iTNTPiston/txtpp)
![Issue Badge](https://img.shields.io/github/issues/iTNTPiston/txtpp)

A simple-to-use general purpose preprocessor for text files, written in rust.

You can:
- Include another file in the current file, much like C-style `#include`
- Execute a command in the middle of the current file and include the output

You can use `txtpp` both as a command line tool, or as a library with the rust crate. `txtpp` is well tested with unit tests and integration tests.

**Currently in Preview. Not all features are implemented and some examples may not work.**

# Installation
Install with `cargo`
```
cargo install txtpp
```

# Examples
## Include a file
Say you have 2 files `foo.txt.txtpp` and `bar.txt`
```
(foo.txt.txtpp)                    (bar.txt)

  1 |hello                           1 |bar
  2 |-TXTPP#include bar.txt          2 |
  3 |world
  4 | 
```

Running `txtpp foo.txt` will produce `foo.txt`:
```
(foo.txt)

  1 |hello
  2 |bar
  3 |world
  4 |
```
If `bar.txt.txtpp` also exists, it will be preprocessed first to produce `bar.txt`, and the result will be used.

## Execute a command
Say you have the file `foo.txt.txtpp`:
```
(foo.txt.txtpp)                   

  1 |hello                           
  2 |-TXTPP#run cat foo.txt.txtpp           
  3 |world
  4 | 
```
Running `txtpp foo.txt` will produce `foo.txt`:
```
(foo.txt)

  1 |hello
  2 |hello
  3 |-TXTPP#run cat foo.txt.txtpp
  3 |world
  4 |world
  5 |
```

More examples can be found in the [examples](examples) directory. These are also used as integration tests.

# Table of Contents
- [Feature Summary](#feature-summary)
- [Directives](#directives)
  - [Syntax](#syntax)
  - [Output](#output)
  - [Specification](#specification)
    - [Include Directive](#include-directive)
    - [Run Directive](#run-directive)
    - [Temp Directive](#temp-directive)
    - [Tag Directive](#tag-directive)
    - [Write Directive](#write-directive)
- [Output Specification](#output-specification)
- [API Doc](https://docs.rs/txtpp)
- [Command Line Interface](#command-line-interface)

# Feature Summary
`txtpp` provides directives that you can use in the `.txtpp` files.
A directive replaces itself with the output of the directive.
The directives are all prefixed with `TXTPP#`:
- `include` - Include the content of another file.
- `run` - Run a command and include the output of the command.
- `temp` - Store text into a temporary file next to the input file.
- `tag` - Hold the output of the next directive until a tag is seen, and replace the tag with the output.
- `write` - Write content to the output file. Can be used for escaping directives. 

# Directives
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
1. Subsequent lines: Some directives are allowed to have more than one lines (see [specification](#specification) for what they are)
    - `{WHITESPACES}`: this must match exactly with `{WHITESPACES}` in the first line
    - `{PREFIX2}`: this must be one of:
      - the same number of space characters (i.e `' '`) as the `{PREFIX1}` in the first line
      - the exact same string as the `{PREFIX1}` in the first line
      - the same string as the `{PREFIX1}` in the first line without trailing whitespaces, followed by new line (i.e. arg is empty)
    - `{ARG2}`, `{ARG3}` ...: Add more arguments to the argument list. Note that `{WHITESPACES}` and `{PREFIX2}` are not included in the argument. Unlike the first line, leading whitespaces are not trimmed, but trailing whitespaces are still trimmed.
1. Ending: if a line does not match the format above, the directive ends. The ending line won't be part of the directive and will still be in the processed file.


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

           TXTPP# */
```
This will execute the command `echo "hello world"`. Note that because `run` supports multi-line, we used an empty line to end the first directive, and then used the empty directive to remove the end of the block comment.

## Output
If the directive has output like `include` and `run`, it will be processed as follows:
- Every line in the output will be prepended with `{WHITESPACES}`, so that the indentation is consistent
- The directive line (or all directive lines, if the directive is multi-line) will not be in the output
  - The `tag` directive can be useful if you have to include the output inline with other text


## Specification
This section contains detailed specification of each directive.

### Include Directive
#### USAGE
This directive is used include the content of another file into the current file.
#### ARGUMENTS
Single-line only. The argument is `FILE_PATH`
#### BEHAVIOR
- If `FILE_PATH` is an absolute path, it will be used as is. Otherwise, it should be relative to the (directory of) the current file.
- If `FILE_PATH` does not end in `.txtpp`, and `FILE_PATH.txtpp` exists, `FILE_PATH.txtpp` will be preprocessed first to produce `FILE_PATH`, and the result will be used as the output. Note that you would still include `FILE_PATH`, not `FILE_PATH.txtpp`.
- If there is an error reading or preprocessing `FILE_PATH`, preprocessing will be aborted.
#### EXAMPLE
```
TXTPP#include foo.txt
```

### Run Directive
#### USAGE
This directive is used to run a command and include the output of the command into the current file.
#### ARGUMENTS
Can have more than one line. The arguments are joined with a single space in between to form the `COMMAND`
#### BEHAVIOR
- The `COMMAND` will be executed as a sub-process.
- Default shell selection:
  - Windows: `powershell -c COMMAND` or `cmd /C COMMAND` if `powershell` is not available
  - Other OS: `sh -c COMMAND`
  - You can override this with a [command line argument]()
- The working directory of the sub-process will be the directory of the current file.
- The sub-process will inherit the environment variables of the main process, with additional environment variables:
  - `TXTPP_FILE`: the absolute path to the current file being processed.
  - (that's the only environment variable for now)
#### EXAMPLE
```
TXTPP#run echo "hello world"
```
#### CAVEATS
1. `txtpp` will not run as a subcommand to avoid processing loops. You shouldn't need to run `txtpp` inside `txtpp` anyway.

### Empty directive
#### USAGE
Empty directive has the empty string as the name and does nothing. It can be used to remove lines from the input.
#### ARGUMENTS
Can have more than one line. All arguments to the empty directive will be ignored.
#### BEHAVIOR
Nothing
#### EXAMPLE
For example, you can write something like:
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run
       ./codegen

     TXTPP# */
}
```

If you have to put the end of the block comment in a new line, make sure to format it correctly so it is treated as part of the directive.
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run
       ./codegen

     TXTPP#
     */
}
```
In both scenarios, the entire block comment `/**/` will be replaced with the output from running `./codegen`

### Temp Directive
#### USAGE
This directive is used to create a temporary file that can be referenced by `run`
#### ARGUMENTS
Must have at least 1 argument. The first argument specifies the `FILE_PATH` to save the output. The rest of the arguments are joined by [line endings](#line-endings) to form the `CONTENT`, with a trailing line ending.
#### BEHAVIOR
- `FILE_PATH` is resolved the same way as the [include directive](#include-directive)
- `FILE_PATH` cannot end in `.txtpp`. It will cause an error.
  - This is to avoid undefined behavior as the preprocessor may or may not pick up the file generated by the `temp` directive.
- `CONTENT` will be saved to `FILE_PATH`
- `FILE_PATH` will be deleted when the current file is done being processed.
#### EXAMPLE
```javascript
function get_cities() {
  // GENERATED
  return [
    // TXTPP#temp print_cities.py
    // import csv
    // with open("cities.csv") as f:
    //     reader = csv.reader(f)
    //     for (name, population) in reader:
    //         print(f"{{name: \"{name}\", population: \"{population}\"}},")

    // TXTPP#run python print_cities.py
  ];
}
```
### Tag Directive
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

### Write Directive
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
```
Output
```
the line below will be written to the output file as is
TXTPP#run echo "hello world"
```

# Output Specification
This section specifies details of the output of the preprocessor.
## Line endings
The output files will have consistent line ending with the input files. If the input file has mixed line endings, the output file will have the same line endings as the first line in the input file.

If the input file does not have a line ending, the output file will have the same line ending as the operating system (i.e. `\r\n` on Windows, `\n` on Unix).


# API
The full API doc is available on [docs.rs](https://docs.rs/txtpp)

# Command Line Interface (CLI)
The binary `txtpp` is used for preprocessing the files. It takes 1 or more files or directories as input, and preprocesses all the `.txtpp` files in those directories.

The general usage is:
```
txtpp [OPTIONS] [FILES...]
```
Options (will be preprocessed with txtpp, hehe)

-r: recursive. if a directory is given, it will recursively preprocess all the files in that directory. If not given, only the files in the given directory will be processed and subdirecotires will be ignored

-q: quiet. if given, the tool will not print anything
-v: verbose. the opposite of quiet. if given, the tool will print more information

-j: number of concurrent tasks. if given, files will be processsed on multiple threads. Only faster if you have a lot of files to process, since a file will be processed twice if a dependency is detected

-s: shell. The command given will be split by spaces. if given, the shell to use for `run` directive. If not given, it will use `powershell -c` on windows (or `cmd /C` if not `powershell` is not available) and `sh -c` otherwise. 

--clean: delete `FOO` for each `FOO.txtpp` file if present.

--verify: preprocess the files but check if the output is the same instead of writing the output. This is useful for verifying that the files are up to date in a CI.
