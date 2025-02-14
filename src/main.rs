use clap::Parser;
use std::path::Path;

mod logger;
mod matcher;
mod notes;

use crate::logger::Logger;
use crate::matcher::MatcherSet;
use crate::notes::Notes;

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

    /// Enable debug mode.
    #[arg(long)]
    debug: bool,
}

fn main() {
    let cli = Cli::parse();
    let mut ignore = MatcherSet::new();
    let mut paths = cli.path;
    let logger = Logger { quiet: !cli.debug };

    if paths.is_empty() {
        logger.robot("Using current directory as the default path");
        paths.push(".".to_string());
    }

    if cli.no_color {
        logger.robot("Colored output is disabled");
    }

    let only = cli
        .only
        .unwrap_or("TODO,FIXME,XXX,HACK,BUG,NOTE,REVIEW,OPTIMIZE,DEBUG,IDEA,DEPRECATED".to_string())
        .split(',')
        .map(String::from)
        .collect::<Vec<String>>();

    let ignore_entries: Vec<String> = cli.ignore.unwrap_or_else(|| {
        let noteignore = Path::new(".noteignore");

        let content = if noteignore.exists() {
            logger.robot("Using custom .noteignore file");

            std::fs::read_to_string(noteignore).unwrap_or_default()
        } else {
            logger.robot("Using default .noteignore file");

            IGNORE.to_string()
        };

        content
            .lines()
            .map(str::trim)
            .filter(|line| !line.starts_with('#') && !line.is_empty())
            .map(String::from)
            .collect()
    });

    for pattern in &ignore_entries {
        ignore.add(pattern);
    }

    let notes = Notes {
        no_color: cli.no_color,
        logger,
        only,
        ignore,
        paths,
    };

    notes.run();
}
