//! The `tri` command line interface.
//!
//! The commands described in `docs/reference/cli.md` are not implemented yet. Until they are,
//! this binary reports the project's status rather than pretending to be more than it is.

fn main() {
    println!(
        "triblenka {} — design stage, no commands implemented yet",
        triblenka::version()
    );
    println!();
    println!("  DESIGN.md    the architecture and why it is shaped this way");
    println!("  docs/        the intended v1 API, written before it is built");
    println!("  ROADMAP.md   what is planned, generated from the cairn backlog");
}
