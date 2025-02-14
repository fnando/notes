use clap::Parser;
use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

const IGNORE: &str = include_str!("../.noteignore");

/// A simple command line tool to extract annotations from files.
///
/// ## Marks
///
/// The following annotations will be extracted:
///
///- TODO:       Something to be done
///- FIXME:      Bug or issue that needs fixing
///- XXX:        Warning about problematic code
///- HACK:       Code that needs cleanup
///- BUG:        Known bug or issue
///- NOTE:       Important note about the code
///- REVIEW:     Code that needs review
///- OPTIMIZE:   Code that needs optimization
///- DEBUG:      Debugging-related notes
///- IDEA:       Suggestion for improvement
///- DEPRECATED: Code scheduled for removal
///
/// ## Ignoring paths
///
/// By default, these are the ignored entries:
///
/// - .git: The git directory
/// - tmp: Temporary files
/// - log: Log directory
/// - *.log: Log files
///
/// If you provide a custom ignore pattern, the default ones will be ignored.
///
/// If you have a .noteignore file in your project, the patterns will be read from it.
/// You must provide one pattern per line.
///
/// ## Glob Syntax
///
/// - `dir` or `file.ext`: match any directory or file named `dir` or `file.ext`, in any level.
/// - `dir/`: match any directory named `dir`, e.g. matches `dir/file.ext`, but not `src/dir/file.ext`.
/// - `/dir/`: match any directory named `dir`, e.g. matches both `dir/file.ext` and `src/dir/file.ext`.
/// - `**/*.ext`: match any file with that given extension, e.g. matches `dir/file.ext`, `dir/foo/bar/file.ext`, `src/dir/file.ext`.
/// - `**/*`: match all files.
/// - `dir/**/*.ext`: match all files with extension `ext` inside `dir`.
/// - `/**/dir/*.ext`: match all files with extension `ext` inside `dir` at any level.
/// - `**/*.{ext1,ext2}`: match all files with extension `ext1` or `ext2`.
/// - `**/*.[jt]s`: match all files with extension `.js` or `.ts`.
#[derive(Parser)]
#[command(version, about, long_about, verbatim_doc_comment)]
struct Cli {
    /// The path to a file or directory. Can be a glob pattern like `**/*.rs`.
    path: Vec<String>,

    /// Filter by type. Multiple types can be provided by separating them with a comma.
    /// By default, all annotations will be returned.
    #[arg(short, long)]
    only: Option<String>,

    /// Ignore files or directories by name. It should be a glob pattern.
    /// Can be set multiple times for different patterns.
    /// The following entries will be always ignored: .git/, tmp/, log/, binaries.
    #[arg(short, long)]
    ignore: Option<Vec<String>>,

    /// Disable color.
    #[arg(long)]
    no_color: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut ignore = MatcherSet::new();
    let marker_matcher = Regex::new(
        r"\b(?<marker>TODO|FIXME|XXX|HACK|BUG|NOTE|REVIEW|OPTIMIZE|DEBUG|IDEA|DEPRECATED)\b:?(?<text>.*?)$",
    ).unwrap();

    let only = cli
        .only
        .unwrap_or("TODO,FIXME,XXX,HACK,BUG,NOTE,REVIEW,OPTIMIZE,DEBUG,IDEA,DEPRECATED".to_string())
        .split(',')
        .map(String::from)
        .collect::<Vec<String>>();

    let entries: Vec<String> = cli.ignore.unwrap_or_else(|| {
        IGNORE
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(String::from)
            .collect()
    });

    for pattern in &entries {
        ignore.add(pattern);
    }

    let current_dir = std::env::current_dir().unwrap();
    let mut notes_count = 0;
    let mut ignored_count = 0;

    cli.path
        .iter()
        .filter_map(expand_to_path_glob)
        .for_each(|p| {
            for path in &expand_glob_to_files(&p, &ignore) {
                let Ok(file) = File::open(path) else { continue };

                for (index, line) in BufReader::new(file).lines().enumerate() {
                    let line = line.unwrap_or_default();
                    let line = line.trim();

                    if let Some(matches) = marker_matcher.captures(line) {
                        let marker = matches.name("marker").unwrap().as_str().trim();
                        let text = truncate(matches.name("text").unwrap().as_str().trim(), 80);
                        let text: String = text.chars().take(60).collect();

                        if text.len() <= 2 {
                            continue;
                        }

                        notes_count += 1;

                        if !only.contains(&marker.to_string()) {
                            ignored_count += 1;
                            continue;
                        }

                        let lineno = index + 1;
                        let relative = path
                            .strip_prefix(&current_dir)
                            .unwrap_or(path)
                            .to_str()
                            .unwrap();
                        let relative = format!("{relative}:{lineno}");

                        let colored_marker = format!("\x1b[33m[{marker}]\x1b[0m");
                        let colored_relative = format!("\x1b[37m{relative}\x1b[0m");

                        if cli.no_color {
                            println!("{marker} {text}");
                            println!("{relative}\n");
                        } else {
                            println!("{colored_marker} {text}");
                            println!("{colored_relative}\n");
                        }
                    }
                }
            }

            print!("ℹ️ Found {notes_count} notes");
            if ignored_count > 0 {
                print!(" ({ignored_count} ignored)");
            }
            println!();
        });
}

fn expand_to_path_glob(entry: &String) -> Option<String> {
    std::fs::canonicalize(entry).ok().map(|path| {
        if path.is_dir() {
            format!("{}/**/*", path.to_str().unwrap())
        } else {
            path.to_str().unwrap().to_string()
        }
    })
}

fn expand_glob_to_files(pattern: &str, ignore: &MatcherSet) -> Vec<PathBuf> {
    glob(pattern)
        .expect("Failed to read glob pattern")
        .filter_map(Result::ok)
        .filter(|p| p.is_file())
        .filter(|p| {
            if ignore.matches(p.to_str().unwrap()) {
                return false;
            }

            if p.is_dir() {
                return false;
            }

            !is_executable(p) && !is_binary(p)
        })
        .collect()
}

fn is_binary(path: &Path) -> bool {
    if let Ok(mut file) = File::open(path) {
        let mut buffer = [0; 24];

        if let Ok(bytes_read) = file.read(&mut buffer) {
            return buffer[..bytes_read]
                .iter()
                .any(|b| *b == 0 || (b.is_ascii_control() && ![9, 10, 13].contains(b)));
        }
    }

    false
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = fs::metadata(path) {
        return metadata.permissions().mode() & 0o111 != 0;
    }
    false
}

#[cfg(windows)]
fn is_executable(path: &Path) -> bool {
    {
        // Windows checks extension (.exe, .bat, etc)
        if let Some(ext) = path.extension() {
            return matches!(
                ext.to_str(),
                Some("exe") | Some("bat") | Some("cmd") | Some("com")
            );
        }
    }
    false
}

struct Matcher {
    raw_pattern: String,
    pattern: Regex,
}

impl Matcher {
    fn matches(&self, target: &str) -> bool {
        let basename = Path::new(target).file_name().unwrap().to_str().unwrap();

        self.raw_pattern == basename || self.pattern.is_match(target)
    }

    fn new(pattern: &str) -> Matcher {
        let raw_pattern = pattern;

        let mut pattern: String = pattern.to_string();
        pattern = Self::compile(&pattern, r"\+", r"\+");
        pattern = Self::compile(&pattern, r"\(", r"\(");
        pattern = Self::compile(&pattern, r"\|", r"\|");
        pattern = Self::compile(&pattern, r"\)", r"\)");
        pattern = Self::compile(&pattern, r"\{(.*?)\}", "($1)");
        pattern = Self::compile(&pattern, r"\{", r"\{");
        pattern = Self::compile(&pattern, r"\}", r"\}");
        pattern = Self::compile(&pattern, r",", "|");
        pattern = Self::compile(&pattern, r"\.", r"\.");
        pattern = Self::compile(&pattern, r"\*{2}", "DOUBLE_STAR");
        pattern = Self::compile(&pattern, r"\.\*", r".\w{1,}");
        pattern = Self::compile(&pattern, r"\*", r".*?");
        pattern = Self::compile(&pattern, r"DOUBLE_STAR/", "(.*?/)?");
        pattern = Self::compile(&pattern, r"^/", "(.*?/)?");
        pattern = Self::compile(&pattern, r"/$", "(/.*?)?");
        pattern = format!("^{pattern}$");

        Matcher {
            raw_pattern: raw_pattern.to_string(),
            pattern: Regex::new(&pattern)
                .unwrap_or_else(|_| panic!("Expected {pattern} to be a valid regex")),
        }
    }

    fn compile(pattern: &str, from: &str, to: &str) -> String {
        Regex::new(from)
            .unwrap_or_else(|_| panic!("Expected {pattern} to be a valid regex"))
            .replace_all(pattern, to)
            .into()
    }
}

fn truncate(s: &str, max_chars: usize) -> String {
    match s.char_indices().nth(max_chars) {
        None => s.to_string(),
        Some((idx, _)) => {
            let mut truncated = s[..idx].to_string();
            truncated.push_str("...");
            truncated
        }
    }
}

struct MatcherSet {
    matchers: Vec<Matcher>,
}

impl MatcherSet {
    fn new() -> Self {
        MatcherSet {
            matchers: Vec::new(),
        }
    }

    fn add(&mut self, pattern: &str) {
        self.matchers.push(Matcher::new(pattern));
    }

    fn matches(&self, target: &str) -> bool {
        self.matchers.iter().any(|matcher| matcher.matches(target))
    }
}

#[test]
fn test_compile_matcher() {
    assert!(Matcher::new("target").matches("target"));
    assert!(Matcher::new("/target/").matches("target"));
    assert!(Matcher::new("/target/").matches("parent/target"));
    assert!(Matcher::new("/target/").matches("parent/child/target"));
    assert!(Matcher::new("file.txt").matches("file.txt"));
    assert!(Matcher::new("**/file.*").matches("file.txt"));
    assert!(Matcher::new("**/file.*").matches("src/file.rs"));
    assert!(Matcher::new("**/file*").matches("src/file.rs"));
    assert!(Matcher::new("**/file*").matches("src/file"));
    assert!(Matcher::new("**/file*").matches("src/file.txt"));
    assert!(Matcher::new("**/.*").matches("src/.git"));
    assert!(Matcher::new("src/").matches("src/a/b/c/d"));
    assert!(Matcher::new("src/").matches("src/a/b/c/d/e.txt"));
    assert!(!Matcher::new("src/").matches("a/src"));
    assert!(!Matcher::new("src/").matches("a/src/b/c/d/e.txt"));
    assert!(Matcher::new("file.{js,ts}").matches("file.js"));
    assert!(Matcher::new("file.{js,ts}").matches("file.ts"));
    assert!(!Matcher::new("file.{js,ts}").matches("file.rs"));
    assert!(Matcher::new("file.[jt]s").matches("file.js"));
    assert!(Matcher::new("file.[jt]s").matches("file.ts"));
    assert!(!Matcher::new("file.[jt]s").matches("file.rs"));
    assert!(!Matcher::new("file.c+").matches("file.cc"));
    assert!(!Matcher::new("file.(c)").matches("file.c"));
    assert!(Matcher::new("file.(c)").matches("file.(c)"));
    assert!(Matcher::new("file.jpe?g").matches("file.jpg"));
    assert!(Matcher::new("file.jpe?g").matches("file.jpeg"));
}

#[test]
fn test_matcher_set() {
    let mut set = MatcherSet::new();
    set.add(".git/");
    set.add("tmp/");

    assert!(set.matches(".git"));
    assert!(set.matches(".git/HEAD"));
    assert!(set.matches("tmp"));
    assert!(set.matches("tmp/server.pid"));
    assert!(!set.matches("dir/tmp"));
}
