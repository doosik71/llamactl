mod cuda_toolkit;
mod huggingface;
mod model_picker;
mod llama_cpp;

fn main() {
    cuda_toolkit::check();
    llama_cpp::check();

    let gguf_list = huggingface::get_text_generation_gguf();

    match model_picker::select_model(&gguf_list) {
        Ok(Some(selected_model)) => {
            if let Err(error) = llama_cpp::run_model(&selected_model) {
                eprintln!("llama-cli execution failed: {error}");
            }
        }
        Ok(None) => {
            if gguf_list.is_empty() {
                println!("No GGUF models for text generation were found.");
            }
        }
        Err(error) => {
            eprintln!("Failed to display the model picker: {error}");
        }
    }
}
