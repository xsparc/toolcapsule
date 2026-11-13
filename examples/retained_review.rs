#[path = "support/retained_review.rs"]
mod workflow;
fn main() -> std::process::ExitCode {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() != 2
        || workflow::run(
            std::path::Path::new(&args[0]),
            std::path::Path::new(&args[1]),
        )
        .is_err()
    {
        eprintln!("{{\"error\":\"review_acceptance_failed\"}}");
        return std::process::ExitCode::from(2);
    }
    println!("{{\"verified\":true,\"guest_execution\":\"none\"}}");
    std::process::ExitCode::SUCCESS
}
