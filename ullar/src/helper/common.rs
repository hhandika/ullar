//! Miscellaneous utility functions
use clap::{crate_description, crate_name, crate_version};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use size::Size;
use std::{fmt::Debug, time::Duration};
use sysinfo::System;

use chrono::Local;
use colored::Colorize;

const LINE_DECORATOR_LEN: usize = 80;
const HEADER_SYMBOL: char = '=';
const FOOTER_SYMBOL: char = '-';

pub fn get_api_version() -> String {
    crate_version!().to_string()
}

pub fn get_timestamp() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

#[cfg(not(tarpaulin_include))]
pub fn init_spinner() -> ProgressBar {
    let spin = ProgressBar::new_spinner();
    let duration: Duration = Duration::from_millis(150);
    spin.enable_steady_tick(duration);
    spin.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("🌑🌒🌓🌔🌕🌖🌗🌘")
            .template("{spinner} {msg}")
            .expect("Failed getting progress bar."),
    );
    spin
}

#[cfg(not(tarpaulin_include))]
pub fn init_progress_bar(len: u64) -> ProgressBar {
    let progress_bar = ProgressBar::new(len);
    let duration: Duration = Duration::from_millis(250);
    progress_bar.enable_steady_tick(duration);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .tick_chars("🌑🌒🌓🌔🌕🌖🌗🌘")
            .template("{spinner} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
            .expect("Failed to set progress bar style"),
    );
    progress_bar
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UllarConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ullar_version: Option<String>,
}

impl Default for UllarConfig {
    fn default() -> Self {
        Self {
            timestamp: None,
            ullar_version: None,
        }
    }
}

impl UllarConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init() -> Self {
        Self {
            timestamp: Some(get_timestamp()),
            ullar_version: Some(get_api_version()),
        }
    }
}

pub struct PrettyHeader {
    len: usize,
    symbol_counts: usize,
}

impl Default for PrettyHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl PrettyHeader {
    pub fn new() -> Self {
        Self {
            len: LINE_DECORATOR_LEN,
            symbol_counts: 0,
        }
    }

    pub fn get_welcome_header(&mut self) {
        let version = crate_version!();
        let app_name = crate_name!();
        let text = format!("{} v{}", app_name, version);
        let text = self.get_header(&text);
        self.len = text.len();
        self.update_symbol_counts();
        log::info!("{}", text.yellow());
        log::info!("{}", crate_description!().yellow());
        log::info!("{}", self.generate_footer_symbols().yellow());
        SystemInfo::new().get_system_info();
    }

    pub fn get_section_header(&mut self, header_text: &str) -> String {
        let mut text = header_text.to_string();
        if header_text.len() < self.len {
            text = self.get_header(header_text);
        }
        self.len = text.len();
        self.update_symbol_counts();
        text.cyan().to_string()
    }

    pub fn get_section_footer(&self) {
        let decorator = self.generate_footer_symbols().cyan().to_string();
        log::info!("{}", decorator);
    }

    fn get_header(&mut self, header_text: &str) -> String {
        let header_len = header_text.len();
        let mut header = String::from(header_text);
        if self.len > header_len {
            self.symbol_counts = (self.len - header_len) / 2;
            header = self.generate_header(header_text);
        }

        header
    }

    fn update_symbol_counts(&mut self) {
        self.symbol_counts = self.len;
    }

    fn generate_header(&mut self, header_text: &str) -> String {
        let decorator = self.generate_symbols(HEADER_SYMBOL);
        let mut header = format!("{} {} {}", decorator, header_text, decorator);
        if header.len() > self.len {
            header.truncate(self.len);
        }
        header
    }

    fn generate_footer_symbols(&self) -> String {
        self.generate_symbols(FOOTER_SYMBOL)
    }

    fn generate_symbols(&self, symbol: char) -> String {
        symbol.to_string().repeat(self.symbol_counts)
    }
}

pub struct SystemInfo {
    info: System,
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu: String,
    pub cores: usize,
    pub threads: usize,
    pub total_memory: u64,
    pub timestamp: String,
}

impl Default for SystemInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfo {
    pub fn new() -> Self {
        Self {
            info: System::new_all(),
            os: System::name().unwrap_or_default(),
            os_version: System::os_version().unwrap_or_default(),
            kernel_version: System::kernel_version().unwrap_or_default(),
            cpu: String::new(),
            cores: 0,
            threads: 0,
            total_memory: 0,
            timestamp: get_timestamp(),
        }
    }

    pub fn get_only_thread_counts() -> usize {
        let info = System::new_all();
        info.cpus().len()
    }

    pub fn get(&mut self) {
        self.get_cpus();
        self.get_cores();
        self.get_thread_count();
        self.get_memory();
    }

    pub fn get_system_info(&mut self) {
        self.get();
        log::info!("{}", "System Information".cyan());
        let os_name = format!("{} v{}", self.os, self.os_version);
        log::info!("{:18}: {}", "OS", os_name);

        log::info!("{:18}: {}", "Kernel Version", self.kernel_version);
        log::info!("{:18}: {}", "CPU", self.cpu);
        log::info!("{:18}: {}", "Physical cores", self.cores);
        log::info!("{:18}: {}", "Threads", self.threads);
        log::info!(
            "{:18}: {:0}",
            "Total Memory",
            Size::from_bytes(self.total_memory).to_string()
        );
        log::info!("{:18}: {}\n", "Timestamp", self.timestamp);
    }

    fn get_thread_count(&mut self) {
        self.threads = self.info.cpus().len()
    }

    fn get_cpus(&mut self) {
        self.cpu = self
            .info
            .cpus()
            .iter()
            .take(1)
            .map(|cpu| cpu.brand())
            .collect::<String>()
    }

    fn get_cores(&mut self) {
        self.cores = self.info.physical_core_count().unwrap_or_default();
    }

    fn get_memory(&mut self) {
        self.total_memory = self.info.total_memory();
    }
}
