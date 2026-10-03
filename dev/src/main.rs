fn main() {
    std::process::exit(
        microlith_dev::run(
            &std::env::args().skip(1).collect::<Vec<_>>(),
            std::path::Path::new("."),
            &[],
            &mut std::io::stderr(),
        )
        .into(),
    );
}
