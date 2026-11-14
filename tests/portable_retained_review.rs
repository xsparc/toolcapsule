#[path = "../examples/support/retained_review.rs"]
mod workflow;
#[test]
fn portable_cli_acceptance_and_exclusive_destination() {
    let temp = tempfile::tempdir().unwrap();
    let dest = temp.path().join("review");
    let binary = std::path::Path::new(env!("CARGO_BIN_EXE_toolcapsule"));
    workflow::run(binary, &dest).unwrap();
    let saved = std::fs::read(dest.join("identity.json")).unwrap();
    assert!(workflow::run(binary, &dest).is_err());
    assert_eq!(saved, std::fs::read(dest.join("identity.json")).unwrap());
}
