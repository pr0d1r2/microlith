fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = microlith_dev::run(
        &args,
        std::path::Path::new("."),
        &mut std::io::stderr(),
    );
    std::process::exit(code.into());
}
