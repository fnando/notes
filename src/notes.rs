use glob::glob;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};

use crate::logger::Logger;
use crate::MatcherSet;

pub struct Notes {
    pub no_color: bool,
    pub logger: Logger,
    pub only: Vec<String>,
    pub ignore: MatcherSet,
    pub paths: Vec<String>,
}

impl Notes {
    pub fn run(&self) {
        let marker_matcher = Regex::new(
            r"\b(?<marker>TODO|FIXME|XXX|HACK|BUG|NOTE|REVIEW|OPTIMIZE|DEBUG|IDEA|DEPRECATED)\b:?(?<text>.*?)$",
        ).unwrap();
        let current_dir = std::env::current_dir().unwrap();
        let mut notes_count = 0;
        let mut ignored_count = 0;

        self.paths
            .iter()
            .filter_map(expand_to_path_glob)
            .for_each(|p| {
                for path in &self.expand_glob_to_files(&p) {
                    let Ok(file) = File::open(path) else {
                        self.logger.warning(&format!("Unable to read {path:?}"));

                        continue;
                    };

                    for (index, line) in BufReader::new(file).lines().enumerate() {
                        let line = line.unwrap_or_default();
                        let line = line.trim();
                        let lineno = index + 1;

                        if let Some(matches) = marker_matcher.captures(line) {
                            let marker = matches.name("marker").unwrap().as_str().trim();
                            let text = truncate(matches.name("text").unwrap().as_str().trim(), 80);
                            let text: String = text.chars().take(60).collect();

                            if text.len() <= 2 {
                                self.logger.robot(
                                &format!("Ignoring note because it's too short: {path:?} (line: {lineno})")
                            );
                                continue;
                            }

                            notes_count += 1;

                            if !self.only.contains(&marker.to_string()) {
                                self.logger.robot(&format!("Ignoring note because it's too short: {path:?}"));
                                ignored_count += 1;
                                continue;
                            }

                            let relative = path
                                .strip_prefix(&current_dir)
                                .unwrap_or(path)
                                .to_str()
                                .unwrap();
                            let relative = format!("{relative}:{lineno}");

                            let colored_marker = format!("\x1b[33m[{marker}]\x1b[0m");
                            let colored_relative = format!("\x1b[37m{relative}\x1b[0m");

                            if self.no_color {
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

    fn expand_glob_to_files(&self, pattern: &str) -> Vec<PathBuf> {
        glob(pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .filter(|p| p.is_file())
            .filter(|p| {
                if self.ignore.matches(p.to_str().unwrap()) {
                    self.logger
                        .robot(&format!("Ignoring path because it's ignored': {p:?}"));
                    return false;
                }

                if p.is_dir() {
                    self.logger
                        .robot(&format!("Ignoring path because it's a directory: {p:?}"));
                    return false;
                }

                if is_binary(p) {
                    self.logger
                        .robot(&format!("Ignoring file because it's a binary: {p:?}"));
                    return false;
                }

                true
            })
            .collect()
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

fn expand_to_path_glob(entry: &String) -> Option<String> {
    std::fs::canonicalize(entry).ok().map(|path| {
        if path.is_dir() {
            format!("{}/**/*", path.to_str().unwrap())
        } else {
            path.to_str().unwrap().to_string()
        }
    })
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
