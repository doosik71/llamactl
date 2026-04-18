use std::io::{self, Write};
use std::process::{Command, Stdio};

pub fn check() {
    match cuda_toolkit_version() {
        Some(version) => {
            println!("CUDA Toolkit is installed.");
            println!("Version: {version}");
        }
        None => {
            println!("CUDA Toolkit is not installed.");

            let install_plan = install_plan();
            println!("Installation command:");
            println!("{}", install_plan.message);

            if let Some(command) = install_plan.command {
                if ask_yes_no("Install CUDA Toolkit? [y/N]: ") {
                    match run_install_command(command) {
                        Ok(()) => println!("Installation command executed."),
                        Err(error) => eprintln!("Installation command failed: {error}"),
                    }
                } else {
                    println!("Skipping installation.");
                }
            } else {
                println!("No automatic install command could be determined. Please check the installation command for your distribution.");
            }
        }
    }
}

fn cuda_toolkit_version() -> Option<String> {
    let output = Command::new("nvcc").arg("--version").output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    parse_cuda_version(&text)
}

fn parse_cuda_version(output: &str) -> Option<String> {
    let release_marker = "release ";
    let start = output.find(release_marker)? + release_marker.len();
    let rest = &output[start..];
    let end = rest.find(',').unwrap_or(rest.len());
    Some(rest[..end].trim().to_string())
}

struct InstallPlan {
    message: &'static str,
    command: Option<&'static str>,
}

fn install_plan() -> InstallPlan {
    if command_exists("apt", &["--version"]) {
        InstallPlan {
            message: "sudo apt update && sudo apt install -y nvidia-cuda-toolkit",
            command: Some("sudo apt update && sudo apt install -y nvidia-cuda-toolkit"),
        }
    } else if command_exists("dnf", &["--version"]) {
        InstallPlan {
            message: "sudo dnf install -y cuda-toolkit",
            command: Some("sudo dnf install -y cuda-toolkit"),
        }
    } else if command_exists("yum", &["--version"]) {
        InstallPlan {
            message: "sudo yum install -y cuda-toolkit",
            command: Some("sudo yum install -y cuda-toolkit"),
        }
    } else if command_exists("pacman", &["-V"]) {
        InstallPlan {
            message: "sudo pacman -S --noconfirm cuda",
            command: Some("sudo pacman -S --noconfirm cuda"),
        }
    } else if command_exists("zypper", &["--version"]) {
        InstallPlan {
            message: "sudo zypper install -y cuda-toolkit",
            command: Some("sudo zypper install -y cuda-toolkit"),
        }
    } else if command_exists("brew", &["--version"]) {
        InstallPlan {
            message: "brew install --cask cuda",
            command: Some("brew install --cask cuda"),
        }
    } else {
        InstallPlan {
            message: "Unable to determine an installation command automatically. Please check the CUDA Toolkit install command for your distribution.",
            command: None,
        }
    }
}

fn command_exists(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

fn ask_yes_no(prompt: &str) -> bool {
    print!("{prompt}");
    if io::stdout().flush().is_err() {
        return false;
    }

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_err() {
        return false;
    }

    matches!(input.trim().to_lowercase().as_str(), "y" | "yes" | "예" | "ㅇ")
}

fn run_install_command(command: &str) -> io::Result<()> {
    let status = Command::new("sh").arg("-c").arg(command).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Exit code: {status}"),
        ))
    }
}
