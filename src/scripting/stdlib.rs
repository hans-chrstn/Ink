use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::process::Command;
use std::thread;

use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug)]
pub enum StdLibError {
    Io(io::Error),
    Reqwest(reqwest::Error),
    Command(String),
}

impl fmt::Display for StdLibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StdLibError::Io(e) => write!(f, "I/O error: {}", e),
            StdLibError::Reqwest(e) => write!(f, "Request error: {}", e),
            StdLibError::Command(e) => write!(f, "Command execution error: {}", e),
        }
    }
}

impl StdError for StdLibError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            StdLibError::Io(e) => Some(e),
            StdLibError::Reqwest(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for StdLibError {
    fn from(err: io::Error) -> Self {
        StdLibError::Io(err)
    }
}

impl From<reqwest::Error> for StdLibError {
    fn from(err: reqwest::Error) -> Self {
        StdLibError::Reqwest(err)
    }
}

pub fn pango_escape_text(text: &str) -> String {
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

pub fn exec(cmd: &str) -> Result<String, StdLibError> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(StdLibError::Io)?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(StdLibError::Command(err));
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
) -> Result<String, StdLibError> {
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
        .map_err(StdLibError::Reqwest)?
        .text()
        .map_err(StdLibError::Reqwest)
}

pub async fn exec_async(cmd: String) -> Result<String, StdLibError> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .map_err(StdLibError::Io)?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(StdLibError::Command(err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn fetch_async(
    method: String,
    uri: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, StdLibError> {
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
        .map_err(StdLibError::Reqwest)?
        .text()
        .await
        .map_err(StdLibError::Reqwest)
}

pub fn markdown_to_pango(markdown: &str) -> String {
    let parser = Parser::new(markdown);
    let mut converter = PangoConverter::new();

    for event in parser {
        converter.process_event(event);
    }

    converter.into_string()
}

struct PangoConverter {
    output: String,
    last_output_char_is_whitespace: bool,
}

impl PangoConverter {
    fn new() -> Self {
        PangoConverter {
            output: String::new(),
            last_output_char_is_whitespace: true,
        }
    }

    fn push_str_and_update_ws(&mut self, s: &str) {
        if !s.is_empty() {
            self.output.push_str(s);
            self.last_output_char_is_whitespace = s.ends_with(char::is_whitespace);
        }
    }

    fn push_char_and_update_ws(&mut self, c: char) {
        self.output.push(c);
        self.last_output_char_is_whitespace = c.is_whitespace();
    }

    fn process_event(&mut self, event: Event) {
        let prepend_space_conditional =
            !self.output.is_empty() && !self.last_output_char_is_whitespace;

        match event {
            Event::Start(Tag::Paragraph) => {
                if !self.output.is_empty() && !self.output.ends_with("\n\n") {
                    self.push_str_and_update_ws("\n\n");
                }
            }
            Event::End(TagEnd::Paragraph) => {
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.push_char_and_update_ws('\n');
                }
            }
            Event::Start(Tag::List(_)) => {
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.push_char_and_update_ws('\n');
                }
            }
            Event::End(TagEnd::List(_)) => {
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.push_char_and_update_ws('\n');
                }
            }
            Event::Start(Tag::Item) => {
                if !self.output.is_empty() && !self.output.ends_with('\n') {
                    self.push_char_and_update_ws('\n');
                }
                self.push_str_and_update_ws("• ");
            }
            Event::End(TagEnd::Item) => {}
            Event::Start(Tag::Strong) => {
                if prepend_space_conditional {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws("<b>");
            }
            Event::End(TagEnd::Strong) => {
                self.push_str_and_update_ws("</b>");
            }
            Event::Start(Tag::Emphasis) => {
                if prepend_space_conditional {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws("<i>");
            }
            Event::End(TagEnd::Emphasis) => {
                self.push_str_and_update_ws("</i>");
            }
            Event::Start(Tag::CodeBlock(_)) => {
                if prepend_space_conditional {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws("<span font_family=\"monospace\">");
            }
            Event::End(TagEnd::CodeBlock) => {
                self.push_str_and_update_ws("</span>");
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                if prepend_space_conditional {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws(&format!(
                    "<a href=\"{}\">",
                    pango_escape_text(&dest_url)
                ));
            }
            Event::End(TagEnd::Link) => {
                self.push_str_and_update_ws("</a>");
            }
            Event::Text(text) => {
                if prepend_space_conditional
                    && !text.starts_with(char::is_whitespace)
                    && !text.starts_with(|c: char| c.is_ascii_punctuation())
                {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws(&pango_escape_text(&text));
            }
            Event::Code(text) => {
                if prepend_space_conditional {
                    self.push_char_and_update_ws(' ');
                }
                self.push_str_and_update_ws(&format!(
                    "<span font_family=\"monospace\">{}</span>",
                    pango_escape_text(&text)
                ));
            }
            Event::SoftBreak => self.push_str_and_update_ws("\n"),
            Event::HardBreak => self.push_str_and_update_ws("<br/>"),
            _ => {}
        }
    }

    fn into_string(self) -> String {
        self.output.trim().to_string()
    }
}
