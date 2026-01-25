use ullar_bwa::bwa::metadata::BwaMetadata;

fn main() {
    let mut bwa = BwaMetadata::new();
    bwa.get();

    print!("BWA Executable: ");
    match &bwa.excutable {
        Some(exe) => println!("{}", exe),
        None => println!("Not found"),
    }
    print!("BWA Version: ");
    match &bwa.version {
        Some(ver) => println!("{}", ver),
        None => println!("Unknown"),
    }
}
