use std::collections::HashMap;
use std::process::Command;
use std::thread;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

fn pango_escape_text(text: &str) -> String {
    let mut escaped = String::new();
    for c in text.chars() {
        match c {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '\'' => escaped.push_str("&apos;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

pub fn exec(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Error: {}", err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn spawn(cmd: String) {
    thread::spawn(move || {
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd);

        match command.spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("Failed to spawn command: {}", e);
            }
        }
    });
}

pub fn fetch(
    method: &str,
    uri: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let mut request = match method.to_lowercase().as_str() {
        "post" => client.post(uri),
        "put" => client.put(uri),
        "delete" => client.delete(uri),
        _ => client.get(uri),
    };

    if let Some(h) = headers {
        for (key, val) in h {
            request = request.header(&key, &val);
        }
    }

    if let Some(b) = body {
        request = request.body(b);
    }

    request
        .send()
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())
}

pub async fn exec_async(cmd: String) -> Result<String, String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Error: {}", err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn fetch_async(
    method: String,
    uri: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut request = match method.to_lowercase().as_str() {
        "post" => client.post(&uri),
        "put" => client.put(&uri),
        "delete" => client.delete(&uri),
        _ => client.get(&uri),
    };

    if let Some(h) = headers {
        for (key, val) in h {
            request = request.header(&key, &val);
        }
    }

    if let Some(b) = body {
        request = request.body(b);
    }

    request
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

pub fn markdown_to_pango(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut pango_output = String::new();

    let mut prev_char_was_whitespace = true;

    for event in parser {
        let prepend_space =
            !pango_output.is_empty() && !pango_output.ends_with(char::is_whitespace);

        match event {
            Event::Start(Tag::Paragraph) => {
                if !pango_output.is_empty() {
                    pango_output.push_str("\n\n");
                }
                prev_char_was_whitespace = true;
            }
            Event::End(TagEnd::Paragraph) => {
                if !pango_output.ends_with('\n') && !pango_output.is_empty() {
                    pango_output.push('\n');
                }
                prev_char_was_whitespace = true;
            }
            Event::Start(Tag::Strong) => {
                if prepend_space {
                    pango_output.push(' ');
                }
                pango_output.push_str("<b>");
                prev_char_was_whitespace = false;
            }
            Event::End(TagEnd::Strong) => {
                pango_output.push_str("</b>");
                prev_char_was_whitespace = false;
            }
            Event::Start(Tag::Emphasis) => {
                if prepend_space {
                    pango_output.push(' ');
                }
                pango_output.push_str("<i>");
                prev_char_was_whitespace = false;
            }
            Event::End(TagEnd::Emphasis) => {
                pango_output.push_str("</i>");
                prev_char_was_whitespace = false;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                if prepend_space {
                    pango_output.push(' ');
                }
                pango_output.push_str("<span font_family=\"monospace\">");
                prev_char_was_whitespace = false;
            }
            Event::End(TagEnd::CodeBlock) => {
                pango_output.push_str("</span>");
                prev_char_was_whitespace = false;
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if prepend_space {
                    pango_output.push(' ');
                }
                pango_output.push_str(&format!("<a href=\"{}\">", pango_escape_text(&dest_url)));
                prev_char_was_whitespace = false;
            }
            Event::End(TagEnd::Link) => {
                pango_output.push_str("</a>");
                prev_char_was_whitespace = false;
            }
            Event::Text(text) => {
                if prepend_space
                    && !text.starts_with(char::is_whitespace)
                    && !text.starts_with(|c: char| c.is_ascii_punctuation())
                {
                    pango_output.push(' ');
                }
                pango_output.push_str(&pango_escape_text(&text));
                prev_char_was_whitespace = pango_output.ends_with(char::is_whitespace);
            }
            Event::Code(text) => {
                if prepend_space {
                    pango_output.push(' ');
                }
                pango_output.push_str(&format!(
                    "<span font_family=\"monospace\">{}</span>",
                    pango_escape_text(&text)
                ));
                prev_char_was_whitespace = false;
            }
            Event::SoftBreak => {
                pango_output.push_str("\n");
                prev_char_was_whitespace = true;
            }
            Event::HardBreak => {
                pango_output.push_str("<br/>");
                prev_char_was_whitespace = true;
            }
            _ => {
                prev_char_was_whitespace = false;
            }
        }
    }

    pango_output.trim().to_string()
}
