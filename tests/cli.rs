use assert_cmd::cargo_bin_cmd;

#[test]
fn prints_helloworld() {
    let mut cmd = cargo_bin_cmd!("nori-lint");
    cmd.assert().success().stdout("helloworld\n");
}
