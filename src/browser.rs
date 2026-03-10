use std::process::Command;

/// Open a URL in the system default browser.
/// Spawns the process and returns immediately (does not wait for browser).
pub fn open_url(url: &str) -> Result<(), std::io::Error> {
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd").args(["/C", "start", url]).spawn()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::open_url;

    #[test]
    fn test_open_url_exists() {
        // Verify function is callable (don't actually open a browser in tests)
        let _ = open_url as fn(&str) -> Result<(), std::io::Error>;
    }
}
