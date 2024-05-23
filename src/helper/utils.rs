//! Miscellaneous utility functions
use clap::{crate_description, crate_name, crate_version};
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;
use sysinfo::System;

use chrono::Local;
use colored::Colorize;

const LINE_DECORATOR_LEN: usize = 80;
const BYTE_TO_GB: u64 = 1024 * 1024 * 1024;

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

pub struct PrettyHeader {
    text: String,
    sym: char,
    len: usize,
    text_len: usize,
    sym_len: usize,
}

impl PrettyHeader {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            sym: '=',
            len: LINE_DECORATOR_LEN,
            text_len: 0,
            sym_len: 0,
        }
    }

    pub fn get_welcome_header(&mut self) {
        let version = crate_version!();
        let app_name = crate_name!();
        self.text = format!("{} v{}", app_name, version);
        self.get_len();
        let symbols = self.generate_header_symbols();
        let text = format!("{} {} {}", symbols, self.text, symbols);
        log::info!("{}", text.yellow());
        log::info!("{}", crate_description!().yellow());
        let closing_line = text.len();
        log::info!("{}", self.generate_symbols(closing_line).yellow());
        SystemInfo::new().get_system_info();
    }

    pub fn get_sample_header(&mut self, text: &str) -> String {
        self.text = text.to_string();
        self.get_len();
        if self.text_len > self.len {
            self.text.yellow().to_string()
        } else {
            self.get_header()
        }
    }

    fn get_header(&mut self) -> String {
        let symbols = self.generate_header_symbols();
        let header = format!("Processing {}", self.text);

        format!("{} {} {}", symbols, header, symbols)
            .yellow()
            .to_string()
    }

    fn get_len(&mut self) {
        self.text_len = self.text.len();

        if self.len > self.text_len {
            self.sym_len = (self.len - self.text_len) / 2;
        } else {
            self.sym_len = self.len;
        }
    }

    fn generate_header_symbols(&self) -> String {
        let mut len = self.sym_len;
        if self.text_len % 2 != 0 {
            len += 1;
        }

        self.generate_symbols(len)
    }

    fn generate_symbols(&self, len: usize) -> String {
        self.sym.to_string().repeat(len)
    }
}

pub struct SystemInfo {
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu: String,
    pub cores: usize,
    pub threads: usize,
    pub total_memory: String,
    pub date: String,
}

impl SystemInfo {
    pub fn new() -> Self {
        let info = System::new_all();
        let os = System::name().unwrap_or_default();
        let os_version = System::os_version().unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();
        let cpu = info
            .cpus()
            .iter()
            .take(1)
            .map(|cpu| cpu.brand())
            .collect::<String>();
        let threads = info.cpus().len();
        let cores = info.physical_core_count().unwrap_or_default();
        let total_memory = info.total_memory() / BYTE_TO_GB;
        let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        Self {
            os,
            os_version,
            kernel_version,
            cpu,
            cores,
            threads,
            total_memory: format!("{} GB", total_memory),
            date,
        }
    }

    pub fn get_system_info(&self) {
        log::info!("{}", "System Information".cyan());
        let os_name = format!("{} v{}", self.os, self.os_version);
        log::info!("{:18}: {}", "OS", os_name);

        log::info!("{:18}: {}", "Kernel Version", self.kernel_version);
        log::info!("{:18}: {}", "CPU", self.cpu);
        log::info!("{:18}: {}", "Physical cores", self.cores);
        log::info!("{:18}: {}", "Threads", self.threads);
        log::info!("{:18}: {}", "Total Memory", self.total_memory);
        log::info!("{:18}: {}\n", "Date", self.date);
    }
}
