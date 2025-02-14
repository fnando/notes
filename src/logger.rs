pub struct Logger {
    pub quiet: bool,
}

impl Logger {
    pub fn robot(&self, message: &str) {
        if !self.quiet {
            eprintln!("🤖 {message}");
        }
    }

    pub fn warning(&self, message: &str) {
        if !self.quiet {
            eprintln!("⚠️ {message}");
        }
    }
}
