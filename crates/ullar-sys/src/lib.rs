use colored::Colorize;
use size::Size;
use sysinfo::System;

pub struct SystemInfo {
    info: System,
    pub os: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu: String,
    pub cores: usize,
    pub threads: usize,
    pub total_memory: u64,
    pub available_memory: u64,
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
            available_memory: 0,
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
        self.get_available_memory();
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
        if self.available_memory > 0 {
            log::info!(
                "{:18}: {}",
                "Available Memory",
                Size::from_bytes(self.available_memory).to_string()
            );
        }
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
        self.cores = num_cpus::get_physical();
    }

    fn get_memory(&mut self) {
        self.total_memory = self.info.total_memory();
    }

    fn get_available_memory(&mut self) {
        self.available_memory = self.info.available_memory();
    }
}

pub fn get_system_memory_mb() -> u64 {
    let mut sys_info = SystemInfo::new();
    sys_info.get_memory();
    // Return memory in MB
    sys_info.total_memory / 1024
}
