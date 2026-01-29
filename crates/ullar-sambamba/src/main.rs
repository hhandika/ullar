use std::path::Path;

const LOG_FILE: &str = "ullar-sambamba.log";
fn main() {
    ullar_logger::init_logger(Path::new(LOG_FILE)).expect("Failed to initialize logger");
}
