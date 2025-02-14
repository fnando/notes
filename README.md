# notes

A command-line tool to display markers like `TODO` and `FIXME`.

![GitHub Release](https://img.shields.io/github/v/release/fnando/notes)

## Install

### Homebrew

```console
$ brew install fnando/tap/notes
```

### Others

Download the binary for your architecture from
https://github.com/fnando/notes/releases/latest

## Usage

A simple command line tool to extract annotations from files.

### Marks

The following annotations will be extracted:

- `TODO`: Something to be done
- `FIXME`: Bug or issue that needs fixing
- `XXX`: Warning about problematic code
- `HACK`: Code that needs cleanup
- `BUG`: Known bug or issue
- `NOTE`: Important note about the code
- `REVIEW`: Code that needs review
- `OPTIMIZE`: Code that needs optimization
- `DEBUG`: Debugging-related notes
- `IDEA`: Suggestion for improvement
- `DEPRECATED`: Code scheduled for removal

### Ignoring paths

If you provide a custom ignore pattern, the default ones will be ignored.

If you have a .noteignore file in your project, the patterns will be read from
it. You must provide one pattern per line.

### Glob Syntax

- `dir` or `file.ext`: match any directory or file named `dir` or `file.ext`, in
  any level.
- `dir/`: match any directory named `dir`, e.g. matches `dir/file.ext`, but not
  `src/dir/file.ext`.
- `/dir/`: match any directory named `dir`, e.g. matches both `dir/file.ext` and
  `src/dir/file.ext`.
- `**/*.ext`: match any file with that given extension, e.g. matches
  `dir/file.ext`, `dir/foo/bar/file.ext`, `src/dir/file.ext`.
- `**/*`: match all files.
- `dir/**/*.ext`: match all files with extension `ext` inside `dir`.
- `/**/dir/*.ext`: match all files with extension `ext` inside `dir` at any
  level.
- `**/*.{ext1,ext2}`: match all files with extension `ext1` or `ext2`.
- `**/*.[jt]s`: match all files with extension `.js` or `.ts`.
