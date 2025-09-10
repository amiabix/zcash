//! Logging utilities

use alloc::string::String;
use alloc::vec::Vec;

/// Log level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

/// Log entry
#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: u64,
    pub module: String,
}

/// Logger
pub struct Logger {
    entries: Vec<LogEntry>,
    min_level: LogLevel,
}

impl Logger {
    /// Create a new logger
    pub fn new(min_level: LogLevel) -> Self {
        Self {
            entries: Vec::new(),
            min_level,
        }
    }

    /// Log a message
    pub fn log(&mut self, level: LogLevel, module: &str, message: &str) {
        if level as u8 <= self.min_level as u8 {
            self.entries.push(LogEntry {
                level,
                message: message.to_string(),
                timestamp: Self::get_current_time(),
                module: module.to_string(),
            });
        }
    }

    /// Log error
    pub fn error(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Error, module, message);
    }

    /// Log warning
    pub fn warn(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Warn, module, message);
    }

    /// Log info
    pub fn info(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Info, module, message);
    }

    /// Log debug
    pub fn debug(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Debug, module, message);
    }

    /// Log trace
    pub fn trace(&mut self, module: &str, message: &str) {
        self.log(LogLevel::Trace, module, message);
    }

    /// Get all log entries
    pub fn get_entries(&self) -> &[LogEntry] {
        &self.entries
    }

    /// Get entries by level
    pub fn get_entries_by_level(&self, level: LogLevel) -> Vec<&LogEntry> {
        self.entries.iter().filter(|e| e.level == level).collect()
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Generate log report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Log Report\n");
        report.push_str("==========\n");
        
        for entry in &self.entries {
            report.push_str(&format!("[{}] {} {}: {}\n", 
                Self::format_timestamp(entry.timestamp),
                Self::format_level(entry.level),
                entry.module,
                entry.message));
        }
        
        report
    }

    /// Format timestamp
    fn format_timestamp(timestamp: u64) -> String {
        // Simplified timestamp formatting
        format!("{}", timestamp)
    }

    /// Format log level
    fn format_level(level: LogLevel) -> &'static str {
        match level {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN ",
            LogLevel::Info => "INFO ",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    /// Get current time
    fn get_current_time() -> u64 {
        // Simplified time - in real implementation would use proper time
        1694323200 * 1_000_000 // Mock timestamp in microseconds
    }
}

impl Default for Logger {
    fn default() -> Self {
        Self::new(LogLevel::Info)
    }
}
