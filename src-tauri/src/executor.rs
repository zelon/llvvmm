
pub fn execute(cmd: &str, args: Vec<&str>) -> String {
    std::process::Command::new(cmd)
        .args(args)
        .output()
        .map(|output| {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.to_string()
        })
        .unwrap_or_else(|err| format!("Failed to execute {}: {}", cmd, err))
}