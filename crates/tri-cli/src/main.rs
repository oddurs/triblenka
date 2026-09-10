//! The `tri` command line interface.
//!
//! Only what M0 needs: generate the benchmark fixture, and measure the two kill criteria.

mod bench;
mod bindings;
mod fixture;

fn main() -> std::process::ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("bench-m0") => {
            let count = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(500);
            match bench::run(count) {
                Ok(true) => std::process::ExitCode::SUCCESS,
                Ok(false) => {
                    eprintln!("a kill criterion was not met");
                    std::process::ExitCode::FAILURE
                }
                Err(error) => {
                    eprintln!("bench failed: {error}");
                    std::process::ExitCode::FAILURE
                }
            }
        }
        Some("fixture") => {
            let dir = bench::default_root().join("content");
            let count = args.get(1).and_then(|n| n.parse().ok()).unwrap_or(500);
            match fixture::generate(&dir, count) {
                Ok(()) => {
                    println!("wrote {count} posts to {}", dir.display());
                    std::process::ExitCode::SUCCESS
                }
                Err(error) => {
                    eprintln!("could not write the fixture: {error}");
                    std::process::ExitCode::FAILURE
                }
            }
        }
        _ => {
            println!("triblenka {} — M0 spike", triblenka::version());
            println!();
            println!("  tri bench-m0 [count]   measure the two M0 kill criteria");
            println!("  tri fixture [count]    write the benchmark fixture");
            println!();
            println!("  DESIGN.md   the architecture   ROADMAP.md   what is planned");
            std::process::ExitCode::SUCCESS
        }
    }
}
