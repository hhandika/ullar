use colored::Colorize;
use comfy_table::Table;

pub struct ProcessingTracker {
    pub sample_counts: usize,
    pub success_counts: usize,
    pub failure_counts: usize,
    /// Streaming mean of the runtime for each sample
    pub mean_runtime: f64,
    /// Total runtime for all samples
    /// to be used for calculating the mean runtime
    pub total_runtime: f64,
    pub wait_time: f64,
    pub total_processed: usize,
}

impl ProcessingTracker {
    pub fn new(sample_counts: usize) -> Self {
        Self {
            sample_counts,
            success_counts: 0,
            failure_counts: 0,
            mean_runtime: 0.0,
            total_runtime: 0.0,
            wait_time: 0.0,
            total_processed: 0,
        }
    }

    pub fn update(&mut self, runtime: f64) {
        self.total_runtime += runtime;
        self.total_processed += 1;
        self.mean_runtime = self.total_runtime / self.total_processed as f64;
        self.wait_time = self.mean_runtime * (self.sample_counts - self.total_processed) as f64;
    }

    pub fn finalize(&self) {
        let success_rate = self.success_counts as f64 / self.sample_counts as f64 * 100.0;
        let mut table = self.print_table();
        table.add_row(vec!["Success rate", &format!("{:.2}%", success_rate)]);
        self.add_mean_runtime(&mut table);
        log::info!("\n{}", "Final Summary".cyan());
        log::info!("{}\n", table);
    }

    pub fn print_summary(&self) {
        let mut table = self.print_table();
        let remaining_samples = self.sample_counts - self.total_processed;
        self.add_mean_runtime(&mut table);
        table.add_row(vec![
            "Estimate wait time",
            &format!("{:.2} s", self.wait_time),
        ]);
        log::info!("\n{}", "Run Summary".cyan());
        log::info!("{}", table);
        log::info!(
            "\nRemaining samples: {} / {}\n",
            remaining_samples,
            self.sample_counts
        );
    }

    fn print_table(&self) -> Table {
        let mut table = Table::new();
        table.set_header(vec!["Metric", "Value"]);
        table
            .add_row(vec!["Total samples", &self.sample_counts.to_string()])
            .add_row(vec!["Total processed", &self.total_processed.to_string()])
            .add_row(vec!["Success", &self.success_counts.to_string()])
            .add_row(vec!["Failure", &self.failure_counts.to_string()]);

        table
    }

    fn add_mean_runtime(&self, table: &mut Table) {
        table.add_row(vec!["Mean runtime", &format!("{:.2} s", self.mean_runtime)]);
    }
}
