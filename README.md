# txtpp
A (really) simple-to-use general purpose preprocessor for text files.

You can:
- Include another file in the current file, much like C-style `#include`
- Execute a command in the middle of the current file and include the output

# Examples
## Include a file
If you have 2 files foo.txt.txtpp and bar.txt

foo.txtpp:
```
hello
TXTPP#include bar.txt
world
```
bar.txt:
```
bar
```

Running `txtpp foo.txt.txtpp` will produce `foo.txt`:
```
hello
bar
world
```

You can also `#include` a `.txtpp` file, and the tool will automatically preprocess that file and include the result.

## Execute a command

foo.txtpp:
```
hello
TXTPP#run cat foo.txtpp 
world
```

Running `txtpp foo.txtpp` will produce `foo.txt`:
```
hello
hello
#run cat foo.txtpp 
world
world
```

By default this uses `powershell -c` on windows and `sh -c` otherwise. You can change that from command line

# Features
`txtpp` really only provides 2 simple directives: `include`, `run`. It also provides 2 additional directives to help with formatting: `nonext` and the empty directive.

## Directive Detection
A directive will be detected if a line looks like this:
```
{WHITESPACES}{SOMETHING}TXTPP#{DIRECTIVE} {STUFF...}
```
Explanation:
- `{WHITESPACES}`: Any number of whitespace characters
- `{SOMETHING}`: Some text that does not start with a whitespace character, and does not include `TXTPP#`
- `TXTPP#`: the prefix before the directive
- `{DIRECTIVE}`: can be one of the directives
- ` `: (space) At least one space between the directive name and its input. This will be trimmed.
- `{STUFF...}`: arguments given to the preprocessor as one string, until the end of the line, without the new line character.

For example, you can write the first line directive like
```
        <!-- TXTPP#include foo.html
```
which will be treated like a comment in HTML to help your editor color things correctly.

## Multi-line directives
If the last character before the newline (`\r\n` or `\n`) is `\`, the next line will also be included in the string that's passed into the directive.

To help with formatting, the next line will be processed as:
```
{WHITESPACES}{SPACING}{CONTENT}
```
where:
- `{WHITESPACES}`: this must match exactly with `{WHITESPACES}` in the first line
- `{SPACING}`: this must be the same number of space characters (" ") as the `{SOMETHING}` in the first line
- `{CONTENT}`: the content for the next line

For example:
```
    // TXTPP#run foo bar \
	     biz
```
will be treated as:
```
    // TXTPP#run foo bar   biz
```
Note that there are 3 spaces between `bar` and `biz`.

If the next line does not match this format, the whole line will be appended to the previous line

## Output Injection
When the preprocessor has the output ready, it will inject the output in the following way:
- Every line in the output will have `{WHITESPACES}` injected, so that the indentation is consistent
- The directive line (or all directive lines, if the directive is multi-line) will not be in the output

## Directive Specs
Each directive takes exactly 1 string as input, and produces 1 string as output (may have newline characters)

### `include FILE`
This directive finds the file specified by `FILE` and use its content as the output. If it is a relative path, it will be treated as relative to the directory the current file being preprocessed is in.

However, if `FILE` does not end in `.txtpp` and `FILE.txtpp` is present, `FILE.txtpp` will be preprocessed first to produce `FILE`, and the result will be used as the output. 

Note that you should include the file without the `.txtpp` extension. If you `TXTPP#include foo.txtpp`, it will literally include `foo.txtpp`. (It will not look for `foo.txtpp.txtpp`)

If `FILE` or `FILE.txtpp` is not found, or the preprocessor fails to process the file, it will abort the preprocessing and exit with error

### `run COMMAND`
This directive executes `COMMAND`. On windows, this translates to `powershell -c COMMAND`. On other OS, it translates to `sh -c COMMAND`. You can use a command line argument to set change the shell to something else.

The working directory will be the directory the current file being preprocessed is in.

There will be some environment variable set before executing the command:
- `TXTPP_FILE`: the path to the current file being processed, relative to the working directory of the current process.
- `TXTPP_DIRECTIVE_INDEX`: the index for the directive in the file. For example, the first directive is `"0"`, the second is "1", etc.

This is useful if you have a custom program that needs to know which file is being processed.

Note that `txtpp` will not run as a subcommand to avoid processing loops. You shouldn't need to run `txtpp` inside `txtpp` anyway.

### `nonext` and empty directive
These directives ignore their input. `nonext` will also eat the next line while the empty directive does nothing. This is helpful with formatting directive as block comments.

For example, you can write something like:
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run \
       ./codegen
     TXTPP# */
}
```

or for some reason you must put the ending on its own line:
```javascript
function hello() {
  // GENERATED CODE
  /* TXTPP#run \
       ./codegen
     TXTPP#nonext
  */
}
```

# Command Line Interface (CLI)
The binary `txtpp` is used for preprocessing the files. It takes 1 or more files or directories as input, and preprocesses all the `.txtpp` files in those directories.

The general usage is:
```
txtpp [OPTIONS] [FILES...]
```
Options (will be preprocessed with txtpp, hehe)

-r: recursive. if a directory is given, it will recursively preprocess all the files in that directory. If not given, only the files in the given directory will be processed and subdirecotires will be ignored

-q: quiet. if given, the tool will not print anything to stdout

-p: parallel. if given, files will be processsed on multiple threads. Only faster if you have a lot of files to process. This will also save a graph of the dependency to `.txtppcache` between files to speed up subsequent runs.

-s: shell. The command given will be split by spaces. if given, the shell to use for `run` directive. If not given, it will use `powershell -c` on windows and `sh -c` otherwise. 

--clean: delete `FOO` for each `FOO.txtpp` file if present.

--verify: preprocess the files but don't write the output. This is useful for verifying that the files are valid.
