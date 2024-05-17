//! Miscellaneous utility functions
use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

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
