//! Performance profiling utilities

use crate::core::*;
use alloc::vec::Vec;
use alloc::string::String;

/// Performance profiler
pub struct PerformanceProfiler {
    start_time: u64,
    measurements: Vec<PerformanceMeasurement>,
}

/// Performance measurement
#[derive(Debug, Clone)]
pub struct PerformanceMeasurement {
    pub name: String,
    pub duration_us: u64,
    pub memory_usage: usize,
    pub cycles: u64,
}

impl PerformanceProfiler {
    /// Create a new performance profiler
    pub fn new() -> Self {
        Self {
            start_time: Self::get_current_time(),
            measurements: Vec::new(),
        }
    }

    /// Start timing
    pub fn start_timing(&mut self) {
        self.start_time = Self::get_current_time();
    }

    /// End timing and record measurement
    pub fn end_timing(&mut self, name: &str, memory_usage: usize, cycles: u64) {
        let end_time = Self::get_current_time();
        let duration_us = end_time - self.start_time;
        
        self.measurements.push(PerformanceMeasurement {
            name: name.to_string(),
            duration_us,
            memory_usage,
            cycles,
        });
    }

    /// Get all measurements
    pub fn get_measurements(&self) -> &[PerformanceMeasurement] {
        &self.measurements
    }

    /// Get total time
    pub fn get_total_time(&self) -> u64 {
        self.measurements.iter().map(|m| m.duration_us).sum()
    }

    /// Get peak memory usage
    pub fn get_peak_memory(&self) -> usize {
        self.measurements.iter().map(|m| m.memory_usage).max().unwrap_or(0)
    }

    /// Get total cycles
    pub fn get_total_cycles(&self) -> u64 {
        self.measurements.iter().map(|m| m.cycles).sum()
    }

    /// Generate performance report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("Performance Report\n");
        report.push_str("==================\n");
        
        for measurement in &self.measurements {
            report.push_str(&format!("{}: {} μs, {} bytes, {} cycles\n", 
                measurement.name, 
                measurement.duration_us, 
                measurement.memory_usage, 
                measurement.cycles));
        }
        
        report.push_str(&format!("Total: {} μs, {} bytes, {} cycles\n", 
            self.get_total_time(), 
            self.get_peak_memory(), 
            self.get_total_cycles()));
        
        report
    }

    /// Get current time in microseconds
    fn get_current_time() -> u64 {
        // Simplified time - in real implementation would use proper time
        1694323200 * 1_000_000 // Mock timestamp in microseconds
    }
}

impl Default for PerformanceProfiler {
    fn default() -> Self {
        Self::new()
    }
}
