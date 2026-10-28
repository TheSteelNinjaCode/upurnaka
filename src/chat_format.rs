//! A small, text-only wire format for chat formatting. Never accepts HTML.
use rahti::{Result, Validator};
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
    pub code: bool,
    pub link: Option<String>,
    pub color: Option<String>,
    pub highlight: Option<String>,
    pub size: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    pub kind: String,
    pub runs: Vec<Run>,
}

pub fn validate(lines: &[Line], text: &str) -> Result<()> {
    let mut validator = Validator::new();
    let count: usize = lines.iter().map(|line| line.runs.len()).sum();
    let plain = lines
        .iter()
        .map(|line| {
            line.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    let color = |value: &Option<String>| {
        value.as_ref().is_none_or(|value| {
            value.len() == 7
                && value.starts_with('#')
                && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
        })
    };
    let valid = lines.len() <= 200
        && count <= 2000
        && plain.chars().count() <= 2000
        && plain.trim() == text
        && lines.iter().all(|line| {
            matches!(
                line.kind.as_str(),
                "paragraph" | "bullet" | "ordered" | "quote" | "code"
            ) && line.runs.iter().all(|run| {
                !run.text.contains('\n')
                    && color(&run.color)
                    && color(&run.highlight)
                    && run
                        .size
                        .as_ref()
                        .is_none_or(|size| matches!(size.as_str(), "small" | "normal" | "large"))
                    && run.link.as_ref().is_none_or(|link| {
                        link.len() <= 2048
                            && !link.chars().any(char::is_control)
                            && (link.starts_with("https://") || link.starts_with("http://"))
                    })
            })
        });
    validator.check(
        "formatting",
        valid,
        "The message contains unsupported formatting.",
    );
    validator.finish()?;
    Ok(())
}
