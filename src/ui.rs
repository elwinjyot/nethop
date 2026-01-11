use std::{
    error::Error,
    io::Write,
    process::{Command, Stdio},
};

pub fn view_in_less(content: &str) -> Result<(), Box<dyn Error>> {
    let mut less = Command::new("less")
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn less: {}", e.to_string()))?;

    if let Some(mut stdin) = less.stdin.take() {
        stdin.write_all(content.as_bytes())?;
    }

    less.wait()?;
    Ok(())
}
