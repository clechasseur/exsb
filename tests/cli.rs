use assert_cmd::{crate_name, Command};

#[test]
fn test_backup_basic() {
    let mut cmd = Command::cargo_bin(crate_name!()).unwrap();

    cmd.arg("backup")
        .arg("--token")
        .arg("SOME_TOKEN")
        .arg("--track")
        .arg("clojure")
        .arg("--track")
        .arg("julia")
        .arg("--exercise")
        .arg("difference-of-squares")
        .arg("--status")
        .arg("published")
        .arg(".")
        .assert()
        .success();
}
