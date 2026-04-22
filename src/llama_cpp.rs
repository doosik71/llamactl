use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::process::Command;

use crate::list_picker::{self, PickerItem};

pub struct CheckOptions {
    pub verbose: bool,
    pub interactive: bool,
}

pub enum ClientInput {
    Prompt(String),
    File(String),
}

pub fn check(options: CheckOptions) -> io::Result<()> {
    if let Some((cli_version, server_version, completion_version)) = llama_cpp_versions() {
        if options.verbose {
            println!("llama.cpp is installed.");
            println!("llama-cli version: {cli_version}");
            println!("llama-server version: {server_version}");
            println!("llama-completion version: {completion_version}");
        }
        return Ok(());
    }

    if options.verbose {
        println!("llama-cli, llama-server, and llama-completion are not all installed.");
    }

    let install_plan = install_plan(options.interactive)?;
    if options.verbose {
        println!("Installation command:");
        println!("{}", install_plan.message);
    }

    let Some(command) = install_plan.command else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No automatic install command could be determined. Please check the llama.cpp install command for your distribution.",
        ));
    };

    if !options.interactive {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "llama-cli, llama-server, and llama-completion are not installed.",
        ));
    }

    if !ask_yes_no("Install llama.cpp? [y/N]: ") {
        if options.verbose {
            println!("Skipping installation.");
        }
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "llama-cli, llama-server, and llama-completion are not installed.",
        ));
    }

    run_install_command(&command)?;
    if options.verbose {
        println!("Installation command executed.");
    }

    match llama_cpp_versions() {
        Some((cli_version, server_version, completion_version)) => {
            if options.verbose {
                println!("llama.cpp installation verified.");
                println!("llama-cli version: {cli_version}");
                println!("llama-server version: {server_version}");
                println!("llama-completion version: {completion_version}");
            }
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "llama.cpp installation completed, but llama-cli, llama-server, and llama-completion are still not available.",
        )),
    }
}

fn llama_cpp_versions() -> Option<(String, String, String)> {
    let cli_version = command_version("llama-cli")?;
    let server_version = command_version("llama-server")?;
    let completion_version = command_version("llama-completion")?;

    Some((cli_version, server_version, completion_version))
}

fn command_version(command: &str) -> Option<String> {
    let output = Command::new(command).arg("--version").output().ok()?;
    let mut text = String::from_utf8_lossy(&output.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stderr));

    extract_version(&text)
}

fn extract_version(output: &str) -> Option<String> {
    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
        let lower = line.to_lowercase();
        if (lower.starts_with("version:") || lower.starts_with("version "))
            && let Some(version) = extract_version_from_line(line)
        {
            return Some(version);
        }
    }

    for line in output
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
    {
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
    message: String,
    command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CudaBuild {
    architectures: String,
    force_cublas: bool,
}

impl CudaBuild {
    fn cmake_flags(&self) -> String {
        let mut flags = vec![format!(
            "-DCMAKE_CUDA_ARCHITECTURES=\"{}\"",
            self.architectures
        )];

        if self.force_cublas {
            flags.push("-DGGML_CUDA_FORCE_CUBLAS=ON".to_string());
        }

        flags.join(" ")
    }
}

fn install_plan(interactive: bool) -> io::Result<InstallPlan> {
    let Some(cuda_build) = resolve_cuda_build(interactive)? else {
        return Ok(InstallPlan {
            message: "No CUDA architecture was selected.".to_string(),
            command: None,
        });
    };

    let extra_cmake_flags = cuda_build.cmake_flags();

    let cmd = format!(
        "if [ -d llama.cpp ]; then \
                cd llama.cpp; \
            else \
                git clone https://github.com/ggerganov/llama.cpp.git && cd llama.cpp; \
            fi && \
            cmake -B build -DLLAMA_SERVER=ON -DGGML_CUDA=ON {extra_cmake_flags} && \
            cmake --build build -j && \
            mkdir -p ~/.local/bin && \
            cp -f \"$(pwd)/build/bin/llama-cli\" ~/.local/bin/llama-cli && \
            cp -f \"$(pwd)/build/bin/llama-server\" ~/.local/bin/llama-server && \
            cp -f \"$(pwd)/build/bin/llama-completion\" ~/.local/bin/llama-completion"
    );

    Ok(InstallPlan {
        message: cmd.clone(),
        command: Some(cmd),
    })
}

fn resolve_cuda_build(interactive: bool) -> io::Result<Option<CudaBuild>> {
    if let Some(detected) = detect_cuda_build() {
        println!(
            "Detected CUDA architecture target: {}",
            detected.architectures
        );
        if detected.force_cublas {
            println!("Using GGML_CUDA_FORCE_CUBLAS=ON for sm_61 stability.");
        }
        return Ok(Some(detected));
    }

    if !interactive {
        return Ok(None);
    }

    println!("Unable to detect CUDA architecture automatically.");
    select_cuda_architecture()
}

fn detect_cuda_build() -> Option<CudaBuild> {
    let output = Command::new("nvidia-smi")
        .args(["--query-gpu=compute_cap", "--format=csv,noheader"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_compute_caps(&stdout)
}

fn parse_compute_caps(output: &str) -> Option<CudaBuild> {
    let mut architectures = BTreeSet::new();
    let mut force_cublas = false;

    for line in output.lines() {
        let compute_cap = line.trim();
        if compute_cap.is_empty() {
            continue;
        }

        let arch = compute_cap_to_architecture(compute_cap)?;
        if arch == "61" {
            force_cublas = true;
        }
        architectures.insert(arch);
    }

    if architectures.is_empty() {
        None
    } else {
        Some(CudaBuild {
            architectures: architectures.into_iter().collect::<Vec<_>>().join(";"),
            force_cublas,
        })
    }
}

fn compute_cap_to_architecture(compute_cap: &str) -> Option<String> {
    let mut parts = compute_cap.split('.');
    let major = parts.next()?.trim();
    let minor = parts.next()?.trim();

    if major.is_empty()
        || minor.is_empty()
        || parts.next().is_some()
        || !major.chars().all(|ch| ch.is_ascii_digit())
        || !minor.chars().all(|ch| ch.is_ascii_digit())
    {
        return None;
    }

    Some(format!("{major}{minor}"))
}

fn select_cuda_architecture() -> io::Result<Option<CudaBuild>> {
    let options = [
        PickerItem {
            display: "61 - Pascal (GTX 1080 and related)".to_string(),
            value: "61".to_string(),
            color: None,
        },
        PickerItem {
            display: "70 - Volta (V100 and related)".to_string(),
            value: "70".to_string(),
            color: None,
        },
        PickerItem {
            display: "75 - Turing (RTX 20xx, T4)".to_string(),
            value: "75".to_string(),
            color: None,
        },
        PickerItem {
            display: "80 - Ampere datacenter (A100)".to_string(),
            value: "80".to_string(),
            color: None,
        },
        PickerItem {
            display: "86 - Ampere consumer (RTX 30xx)".to_string(),
            value: "86".to_string(),
            color: None,
        },
        PickerItem {
            display: "89 - Ada (RTX 40xx)".to_string(),
            value: "89".to_string(),
            color: None,
        },
        PickerItem {
            display: "90 - Hopper (H100)".to_string(),
            value: "90".to_string(),
            color: None,
        },
        PickerItem {
            display: "61;70;75;80;86;89;90 - Common multi-arch build".to_string(),
            value: "61;70;75;80;86;89;90".to_string(),
            color: None,
        },
    ];

    Ok(
        list_picker::select_value(&options, "Select CUDA architecture target:")?.map(
            |architectures| CudaBuild {
                force_cublas: architectures.split(';').any(|arch| arch == "61"),
                architectures,
            },
        ),
    )
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

    matches!(
        input.trim().to_lowercase().as_str(),
        "y" | "yes" | "예" | "ㅇ"
    )
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

#[cfg(test)]
mod tests {
    use super::{CudaBuild, compute_cap_to_architecture, extract_version, parse_compute_caps};

    #[test]
    fn compute_capability_is_converted_to_cmake_architecture() {
        assert_eq!(compute_cap_to_architecture("6.1").as_deref(), Some("61"));
        assert_eq!(compute_cap_to_architecture("8.9").as_deref(), Some("89"));
        assert_eq!(compute_cap_to_architecture("9.0").as_deref(), Some("90"));
    }

    #[test]
    fn multiple_gpus_are_deduplicated_and_sorted() {
        let output = "8.9\n6.1\n8.9\n";
        assert_eq!(
            parse_compute_caps(output),
            Some(CudaBuild {
                architectures: "61;89".to_string(),
                force_cublas: true,
            })
        );
    }

    #[test]
    fn invalid_compute_capability_is_rejected() {
        assert!(compute_cap_to_architecture("not-supported").is_none());
        assert!(parse_compute_caps("6.1\nnot-supported\n").is_none());
    }

    #[test]
    fn force_cublas_is_enabled_for_sm_61() {
        assert_eq!(
            parse_compute_caps("6.1\n"),
            Some(CudaBuild {
                architectures: "61".to_string(),
                force_cublas: true,
            })
        );
    }

    #[test]
    fn version_line_is_preferred_over_compute_capability() {
        let output = "\
ggml_cuda_init: found 1 CUDA devices (Total VRAM: 8110 MiB):\n\
  Device 0: NVIDIA GeForce GTX 1080, compute capability 6.1, VMM: yes, VRAM: 8110 MiB\n\
version: 8886 (17f624516)\n\
built with GNU 13.3.0 for Linux x86_64\n";
        assert_eq!(extract_version(output).as_deref(), Some("8886"));
    }
}

pub fn run_client(model: &str, input: Option<ClientInput>, verbose: bool) -> io::Result<()> {
    if verbose {
        println!("llama-cli -hf {model}");
    }
    let mut command = Command::new("llama-cli");
    command.arg("-hf").arg(model);

    match input {
        Some(ClientInput::Prompt(prompt)) => {
            command.arg("-p").arg(prompt);
        }
        Some(ClientInput::File(file)) => {
            command.arg("-p").arg(read_prompt_file(&file)?);
        }
        None => {}
    }

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Exit code: {status}"),
        ))
    }
}

pub fn run_completion(model: &str, input: Option<ClientInput>, verbose: bool) -> io::Result<()> {
    if verbose {
        println!("llama-completion -hf {model} --single-turn --simple-io --log-disable");
    }
    let mut command = Command::new("llama-completion");
    command.arg("-hf").arg(model);
    command.arg("--no-conversation");
    command.arg("--single-turn");
    command.arg("--simple-io");
    command.arg("--log-disable");

    match input {
        Some(ClientInput::Prompt(prompt)) => {
            command.arg("--prompt").arg(prompt);
        }
        Some(ClientInput::File(file)) => {
            command.arg("--prompt").arg(read_prompt_file(&file)?);
        }
        None => {}
    }

    let status = command.status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Exit code: {status}"),
        ))
    }
}

fn read_prompt_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}

pub fn run_server(model: &str, verbose: bool) -> io::Result<()> {
    if verbose {
        println!("llama-server --webui-mcp-proxy -hf {model}");
    }
    let status = Command::new("llama-server")
        .arg("-hf")
        .arg(model)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Exit code: {status}"),
        ))
    }
}
