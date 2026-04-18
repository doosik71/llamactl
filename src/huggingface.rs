#![allow(dead_code)]

use std::collections::HashSet;
use std::io;
use std::process::Command;

const HF_MODELS_API: &str = "https://huggingface.co/api/models";
const PAGE_SIZE: usize = 100;

pub fn get_text_generation_gguf() -> Vec<String> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let mut next_url = Some(build_query_url());

    while let Some(url) = next_url {
        let (headers, body) = match fetch_url(&url) {
            Ok(response) => response,
            Err(_) => break,
        };

        for model_id in parse_model_ids(&body) {
            if seen.insert(model_id.clone()) {
                results.push(model_id);
            }
        }

        next_url = parse_next_link(&headers);
    }

    results.sort_unstable();
    results
}

fn build_query_url() -> String {
    format!(
        "{HF_MODELS_API}?filter=text-generation&search=gguf&limit={PAGE_SIZE}&full=false&sort=downloads&direction=-1"
    )
}

fn fetch_url(url: &str) -> io::Result<(String, String)> {
    let output = Command::new("curl")
        .args(["-fsSL", "-L", "-D", "/dev/stderr", "-o", "-", url])
        .output()?;

    if output.status.success() {
        let headers = String::from_utf8_lossy(&output.stderr).to_string();
        let body = String::from_utf8_lossy(&output.stdout).to_string();
        Ok((headers, body))
    } else {
        Err(io::Error::new(
            io::ErrorKind::Other,
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

fn parse_next_link(headers: &str) -> Option<String> {
    let mut last_headers = headers;
    if let Some((_, tail)) = headers.rsplit_once("\r\n\r\n") {
        last_headers = tail;
    } else if let Some((_, tail)) = headers.rsplit_once("\n\n") {
        last_headers = tail;
    }

    for line in last_headers.lines() {
        let lower = line.to_ascii_lowercase();
        if !lower.starts_with("link:") {
            continue;
        }

        if let Some(start) = line.find('<') {
            let rest = &line[start + 1..];
            let end = rest.find('>')?;
            let url = rest[..end].trim();
            if line.contains("rel=\"next\"") {
                return Some(url.to_string());
            }
        }
    }

    None
}

fn parse_model_ids(body: &str) -> Vec<String> {
    let mut ids = Vec::new();
    ids.extend(extract_json_string_values(body, "\"modelId\""));
    ids.extend(extract_json_string_values(body, "\"id\""));

    let mut seen = HashSet::new();
    ids.into_iter()
        .filter(|id| seen.insert(id.clone()))
        .collect()
}

fn extract_json_string_values(input: &str, key: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut search_start = 0;

    while let Some(key_index) = input[search_start..].find(key) {
        let key_index = search_start + key_index;
        let after_key = &input[key_index + key.len()..];
        let colon_index = match after_key.find(':') {
            Some(index) => index,
            None => break,
        };
        let after_colon = after_key[colon_index + 1..].trim_start();
        if let Some((value, consumed)) = parse_json_string(after_colon) {
            values.push(value);
            search_start = key_index + key.len() + colon_index + 1 + consumed;
        } else {
            search_start = key_index + key.len();
        }
    }

    values
}

fn parse_json_string(input: &str) -> Option<(String, usize)> {
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if first != '"' {
        return None;
    }

    let mut value = String::new();
    let mut escaped = false;
    for (idx, ch) in chars {
        if escaped {
            value.push(match ch {
                '"' => '"',
                '\\' => '\\',
                '/' => '/',
                'b' => '\u{0008}',
                'f' => '\u{000c}',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }

        match ch {
            '\\' => escaped = true,
            '"' => return Some((value, idx + ch.len_utf8())),
            other => value.push(other),
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_model_ids_from_json() {
        let body = r#"[{"modelId":"Jiunsong/supergemma4-26b-uncensored-gguf-v2"},{"modelId":"Jackrong/Qwen3.5-9B-GLM5.1-Distill-v1-GGUF"},{"id":"unsloth/Qwen3-Coder-Next-GGUF"}]"#;

        let ids = parse_model_ids(body);

        assert_eq!(
            ids,
            vec![
                "Jiunsong/supergemma4-26b-uncensored-gguf-v2".to_string(),
                "Jackrong/Qwen3.5-9B-GLM5.1-Distill-v1-GGUF".to_string(),
                "unsloth/Qwen3-Coder-Next-GGUF".to_string(),
            ]
        );
    }

    #[test]
    fn parses_next_link_header() {
        let headers = "HTTP/2 200\nLink: <https://huggingface.co/api/models?cursor=abc>; rel=\"next\"\n";

        assert_eq!(
            parse_next_link(headers).as_deref(),
            Some("https://huggingface.co/api/models?cursor=abc")
        );
    }
}
