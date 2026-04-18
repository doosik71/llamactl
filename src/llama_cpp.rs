use std::io::{self, Write};
use std::process::Command;

pub fn check() {
    match llama_cpp_versions() {
        Some((cli_version, server_version)) => {
            println!("llama.cpp is installed.");
            println!("llama-cli version: {cli_version}");
            println!("llama-server version: {server_version}");
        }
        None => {
            println!("llama-cli and llama-server are not both installed.");

            let install_plan = install_plan();
            println!("Installation command:");
            println!("{}", install_plan.message);

            if let Some(command) = install_plan.command {
                if ask_yes_no("Install llama.cpp? [y/N]: ") {
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

fn llama_cpp_versions() -> Option<(String, String)> {
    let cli_version = command_version("llama-cli")?;
    let server_version = command_version("llama-server")?;

    Some((cli_version, server_version))
}

fn command_version(command: &str) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    extract_version(&text)
}

fn extract_version(output: &str) -> Option<String> {
    for line in output.lines().map(str::trim).filter(|line| !line.is_empty()) {
        if let Some(version) = extract_version_from_line(line) {
            return Some(version);
        }
    }

    None
}

fn extract_version_from_line(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if let Some(version_idx) = lower.find("version") {
        let after = line[version_idx + "version".len()..].trim();
        return extract_version_after_label(after).or_else(|| first_version_token(after));
    }

    first_version_token(line)
}

fn extract_version_after_label(text: &str) -> Option<String> {
    let trimmed = text.trim_start_matches([':', '=']).trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(first_token) = trimmed.split_whitespace().next() {
        let cleaned = first_token.trim_matches(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        });

        if !cleaned.is_empty() && cleaned.chars().any(|c| c.is_ascii_digit()) {
            return Some(cleaned.trim_start_matches('v').to_string());
        }
    }

    None
}

fn first_version_token(text: &str) -> Option<String> {
    for token in text.split_whitespace() {
        let token = token.trim_matches(|c: char| {
            !(c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
        });

        if looks_like_version(token) {
            return Some(token.trim_start_matches('v').to_string());
        }
    }

    None
}

fn looks_like_version(token: &str) -> bool {
    let has_digit = token.chars().any(|c| c.is_ascii_digit());
    let has_separator = token.contains('.');
    has_digit && has_separator
}

struct InstallPlan {
    message: &'static str,
    command: Option<&'static str>,
}

fn install_plan() -> InstallPlan {
    let cmd = "if [ -d llama.cpp ]; then \
                cd llama.cpp; \
            else \
                git clone https://github.com/ggerganov/llama.cpp.git && cd llama.cpp; \
            fi && \
            cmake -B build -DLLAMA_SERVER=ON && \
            cmake --build build -j && \
            mkdir -p ~/.local/bin && \
            ln -sf \"$(pwd)/build/bin/llama-cli\" ~/.local/bin/llama-cli && \
            ln -sf \"$(pwd)/build/bin/llama-server\" ~/.local/bin/llama-server";

    InstallPlan {
        message: cmd,
        command: Some(cmd),
    }
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

pub fn run_model(model: &str) -> io::Result<()> {
    println!("llama-cli -m {model}");
    let status = Command::new("llama-cli").arg("-hf").arg(model).status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Exit code: {status}"),
        ))
    }
}
