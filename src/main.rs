use std::env;
use std::io;

mod cuda_toolkit;
mod huggingface;
mod list_picker;
mod llama_cpp;
mod model_picker;

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Client,
    Server,
}

struct CliArgs {
    run_mode: Option<RunMode>,
    model: Option<String>,
    ctx_size: Option<u32>,
    prompt: Option<String>,
    file: Option<String>,
    list: bool,
}

fn main() {
    let args = match parse_args() {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };

    if args.list {
        let gguf_list = huggingface::get_text_generation_gguf();

        if gguf_list.is_empty() {
            println!("No GGUF models for text generation were found.");
        } else {
            if let Err(error) = model_picker::print_model_list(&gguf_list) {
                eprintln!("Failed to print model list: {error}");
                std::process::exit(1);
            }
        }
        return;
    }

    if args.prompt.is_some() && args.file.is_some() {
        eprintln!("--prompt and --file cannot be used together.");
        std::process::exit(1);
    }

    if args.run_mode == Some(RunMode::Server) && (args.prompt.is_some() || args.file.is_some()) {
        eprintln!("--prompt and --file can only be used with --mode client.");
        std::process::exit(1);
    }

    let automated = matches!(args.run_mode, Some(RunMode::Client))
        && args.model.is_some()
        && (args.prompt.is_some() || args.file.is_some());

    let check_options = cuda_toolkit::CheckOptions {
        verbose: !automated,
        interactive: !automated,
    };
    if let Err(error) = cuda_toolkit::check(check_options) {
        eprintln!("CUDA Toolkit check failed: {error}");
        std::process::exit(1);
    }

    let check_options = llama_cpp::CheckOptions {
        verbose: !automated,
        interactive: !automated,
    };
    if let Err(error) = llama_cpp::check(check_options) {
        eprintln!("llama.cpp check failed: {error}");
        std::process::exit(1);
    }

    let run_mode = match args.run_mode {
        Some(run_mode) => run_mode,
        None => match select_run_mode() {
            Ok(run_mode) => run_mode,
            Err(error) => {
                eprintln!("Failed to select run mode: {error}");
                std::process::exit(1);
            }
        },
    };

    let selected_model = match args.model {
        Some(model) => model,
        None => {
            let gguf_list = huggingface::get_text_generation_gguf();

            match model_picker::select_model(&gguf_list) {
                Ok(Some(selected_model)) => selected_model,
                Ok(None) => {
                    if gguf_list.is_empty() {
                        println!("No GGUF models for text generation were found.");
                    }
                    return;
                }
                Err(error) => {
                    eprintln!("Failed to display the model picker: {error}");
                    std::process::exit(1);
                }
            }
        }
    };

    let ctx_size = match args.ctx_size {
        Some(size) => size,
        None if automated => llama_cpp::DEFAULT_CONTEXT_SIZE,
        None => match llama_cpp::select_context_size() {
            Ok(Some(size)) => size,
            Ok(None) => return,
            Err(error) => {
                eprintln!("Failed to select context size: {error}");
                std::process::exit(1);
            }
        },
    };

    let client_input = match (args.prompt, args.file) {
        (Some(prompt), None) => Some(llama_cpp::ClientInput::Prompt(prompt)),
        (None, Some(file)) => Some(llama_cpp::ClientInput::File(file)),
        (None, None) => None,
        (Some(_), Some(_)) => unreachable!(),
    };

    let verbose = !automated;
    let result = match run_mode {
        RunMode::Client if automated => {
            llama_cpp::run_completion(&selected_model, client_input, verbose)
        }
        RunMode::Client => llama_cpp::run_client(&selected_model, ctx_size, client_input, verbose),
        RunMode::Server => llama_cpp::run_server(&selected_model, ctx_size, verbose),
    };

    if let Err(error) = result {
        let command_name = match run_mode {
            RunMode::Client if automated => "llama-completion",
            RunMode::Client => "llama-cli",
            RunMode::Server => "llama-server",
        };
        eprintln!("{command_name} execution failed: {error}");
        std::process::exit(1);
    }
}

fn parse_args() -> Result<CliArgs, String> {
    let mut run_mode = None;
    let mut model = None;
    let mut ctx_size = None;
    let mut prompt = None;
    let mut file = None;
    let mut list = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            "--list" => {
                list = true;
            }
            "--mode" => {
                let value = args.next().ok_or("--mode requires a value.")?;
                run_mode = Some(parse_run_mode(&value)?);
            }
            "--model" => {
                let value = args.next().ok_or("--model requires a value.")?;
                model = Some(value);
            }
            "--ctx-size" => {
                let value = args.next().ok_or("--ctx-size requires a value.")?;
                ctx_size = Some(parse_ctx_size(&value)?);
            }
            "--prompt" => {
                let value = args.next().ok_or("--prompt requires a value.")?;
                prompt = Some(value);
            }
            "--file" => {
                let value = args.next().ok_or("--file requires a value.")?;
                file = Some(value);
            }
            _ => {
                return Err(format!("Unknown argument: {arg}"));
            }
        }
    }

    Ok(CliArgs {
        run_mode,
        model,
        ctx_size,
        prompt,
        file,
        list,
    })
}

fn parse_run_mode(value: &str) -> Result<RunMode, String> {
    match value {
        "client" => Ok(RunMode::Client),
        "server" => Ok(RunMode::Server),
        _ => Err(format!("Invalid value for --mode: {value}")),
    }
}

fn print_help() {
    println!(
        "Usage: ezllama [--list] [--mode client|server] [--model <name>] [--ctx-size <size>]"
    );
}

fn parse_ctx_size(value: &str) -> Result<u32, String> {
    let size = value
        .parse::<u32>()
        .map_err(|_| format!("Invalid value for --ctx-size: {value}"))?;

    if llama_cpp::is_supported_context_size(size) {
        Ok(size)
    } else {
        Err(format!(
            "Unsupported value for --ctx-size: {size}. Supported values are 4096, 8192, 16384, and 32768."
        ))
    }
}

fn select_run_mode() -> io::Result<RunMode> {
    let options = [
        list_picker::PickerItem {
            display: "Client (llama-cli)".to_string(),
            value: "client".to_string(),
            color: None,
        },
        list_picker::PickerItem {
            display: "Server (llama-server)".to_string(),
            value: "server".to_string(),
            color: None,
        },
    ];

    let selected = list_picker::select_value(&options, "Select execution mode:")?;

    match selected.as_deref() {
        Some("client") => Ok(RunMode::Client),
        Some("server") => Ok(RunMode::Server),
        Some(other) => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Invalid run mode value: {other}"),
        )),
        None => Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "Run mode selection was canceled.",
        )),
    }
}
