use anyhow::{anyhow, Result};
use std::{env::set_current_dir, path::PathBuf, process::Command};

pub fn patch(patches: Vec<PathBuf>, workdir: &PathBuf) -> Result<()> {
    set_current_dir(workdir)?;
    for patch in patches {
        print!("Applying {}", &patch.display());
        match Command::new("sh")
            .args(["-c", &format!("patch -p1 < {}", &patch.display())])
            .status()
        {
            Ok(status) => {
                if !status.success() {
                    return Err(anyhow!("failed to apply {}", patch.display()));
                }
            }
            Err(e) => {
                eprintln!("{e}");
                return Err(anyhow!("failed to apply {}", patch.display()));
            }
        }
    }
    Ok(())
}
