//! The `mth` binary: a shell over the library, and nothing else.
//!
//! Every rule, every verb and every test lives in the lib, so the format is
//! testable without spawning a process and a consumer can call the same code
//! instead of re-porting it (V7). This file does only the I/O the core
//! avoids: read argv, write the streams, return the code.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let o = microlith::run(&args);
    print!("{}", o.out);
    eprint!("{}", o.err);
    ExitCode::from(o.code)
}
