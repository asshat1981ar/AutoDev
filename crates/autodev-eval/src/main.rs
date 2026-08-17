fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(autodev_eval::run_cli(&args));
}
