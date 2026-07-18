//! cn binary entry: thin shell over the `cn` library router (D-044.2).

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match cn::run(&args, &mut std::io::stdout(), &mut std::io::stderr()) {
        Ok(exit) => exit.into(),
        Err(io_err) => {
            eprintln!("error: stream write failed: {io_err}");
            cn::Exit::Failure.into()
        }
    }
}
