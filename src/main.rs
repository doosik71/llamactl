use std::env;
use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor},
    terminal::{self, Clear, ClearType},
};

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
        RunMode::Client => llama_cpp::run_client(&selected_model, client_input, verbose),
        RunMode::Server => llama_cpp::run_server(&selected_model, verbose),
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
    println!("Usage: ezllama [--list] [--mode client|server] [--model <name>]");
}

fn select_run_mode() -> io::Result<RunMode> {
    let options = [
        ("Client (llama-cli)", RunMode::Client),
        ("Server (llama-server)", RunMode::Server),
    ];

    let selected = list_picker::select_index(options.len(), |stdout, selected, offset| {
        draw_run_mode(stdout, &options, selected, offset)
    })?;

    selected.map(|index| options[index].1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Interrupted,
            "Run mode selection was canceled.",
        )
    })
}

fn draw_run_mode(
    stdout: &mut io::Stdout,
    options: &[(&str, RunMode)],
    selected: usize,
    offset: usize,
) -> io::Result<()> {
    let (_, rows) = terminal::size()?;
    let visible_rows = rows.saturating_sub(2).max(1) as usize;
    let end = (offset + visible_rows).min(options.len());
    let max_width = terminal::size()?.0 as usize;

    queue!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All))?;
    writeln!(stdout, "Select execution mode:")?;

    for (index, (label, _mode)) in options[offset..end].iter().enumerate() {
        let absolute_index = offset + index;
        let y = (index + 1) as u16;
        let prefix = if absolute_index == selected {
            "> "
        } else {
            "  "
        };
        let line = format_line(prefix, label, max_width);

        queue!(stdout, cursor::MoveTo(0, y), Clear(ClearType::CurrentLine))?;

        if absolute_index == selected {
            queue!(
                stdout,
                SetAttribute(Attribute::Reverse),
                SetForegroundColor(Color::Reset),
                Print(line),
                SetAttribute(Attribute::Reset),
                ResetColor
            )?;
        } else {
            queue!(stdout, Print(line))?;
        }
    }

    if rows > 1 {
        queue!(
            stdout,
            cursor::MoveTo(0, rows - 1),
            Clear(ClearType::CurrentLine)
        )?;
        write!(stdout, "↑/↓ to move, Enter to select, Esc to exit")?;
    }

    stdout.flush()?;
    Ok(())
}

fn format_line(prefix: &str, text: &str, max_width: usize) -> String {
    let prefix_width = prefix.chars().count();
    if max_width <= prefix_width {
        return truncate_to_width(prefix, max_width);
    }

    let available = max_width - prefix_width;
    let text_part = truncate_to_width(text, available);
    let mut line = String::with_capacity(prefix.len() + text_part.len());
    line.push_str(prefix);
    line.push_str(&text_part);
    line
}

fn truncate_to_width(text: &str, max_width: usize) -> String {
    if max_width == 0 {
        return String::new();
    }

    let char_count = text.chars().count();
    if char_count <= max_width {
        return text.to_string();
    }

    if max_width == 1 {
        return "…".to_string();
    }

    let mut result = String::new();
    for ch in text.chars().take(max_width.saturating_sub(1)) {
        result.push(ch);
    }
    result.push('…');
    result
}
