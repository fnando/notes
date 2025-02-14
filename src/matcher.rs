use regex::Regex;
use std::path::Path;

pub struct Matcher {
    pub raw_pattern: String,
    pub pattern: Regex,
}

impl Matcher {
    pub fn matches(&self, target: &str) -> bool {
        let basename = Path::new(target).file_name().unwrap().to_str().unwrap();

        self.raw_pattern == basename || self.pattern.is_match(target)
    }

    pub fn new(pattern: &str) -> Matcher {
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

#[allow(clippy::module_name_repetitions)]
pub struct MatcherSet {
    pub matchers: Vec<Matcher>,
}

impl MatcherSet {
    pub fn new() -> Self {
        MatcherSet {
            matchers: Vec::new(),
        }
    }

    pub fn add(&mut self, pattern: &str) {
        self.matchers.push(Matcher::new(pattern));
    }

    pub fn matches(&self, target: &str) -> bool {
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
