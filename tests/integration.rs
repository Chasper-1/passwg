use std::process::Command;

#[test]
fn test_deterministic_mode() {
    let output1 = Command::new("./target/release/passwg")
        .args(&["16", "1", "-S", "testseed"])
        .output()
        .expect("failed to execute");
    let output2 = Command::new("./target/release/passwg")
        .args(&["16", "1", "-S", "testseed"])
        .output()
        .expect("failed to execute");
    assert_eq!(output1.stdout, output2.stdout);
}

#[test]
fn test_different_seed() {
    let output1 = Command::new("./target/release/passwg")
        .args(&["16", "1", "-S", "seed1"])
        .output()
        .expect("failed to execute");
    let output2 = Command::new("./target/release/passwg")
        .args(&["16", "1", "-S", "seed2"])
        .output()
        .expect("failed to execute");
    assert_ne!(output1.stdout, output2.stdout);
}