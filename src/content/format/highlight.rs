use syntect::{
    easy::HighlightLines,
    highlighting::Theme,
    html::{IncludeBackground, styled_line_to_highlighted_html},
    parsing::SyntaxSet,
    util::LinesWithEndings,
};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let themes = syntect::highlighting::ThemeSet::load_defaults();
        let theme = themes
            .themes
            .get("base16-ocean.dark")
            .cloned()
            .unwrap_or_else(|| {
                themes
                    .themes
                    .values()
                    .next()
                    .cloned()
                    .expect("syntect themes should not be empty")
            });

        Self { syntax_set, theme }
    }

    pub fn highlight_code_block(
        &self,
        code: &str,
        language: Option<&str>,
        shell_prompt: bool,
    ) -> String {
        let syntax = language
            .and_then(|lang| self.syntax_set.find_syntax_by_token(lang))
            .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
        let mut highlighter = HighlightLines::new(syntax, &self.theme);
        let mut output = String::new();
        let mut lines = LinesWithEndings::from(code).peekable();

        while let Some(line) = lines.next() {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            let html_line = styled_line_to_highlighted_html(&ranges, IncludeBackground::No)
                .unwrap_or_else(|_| line.to_string());

            if shell_prompt {
                let trimmed = line.trim_end_matches('\n');
                let is_trailing_empty = trimmed.trim().is_empty() && lines.peek().is_none();
                if trimmed.trim().is_empty() && !is_trailing_empty {
                    output.push_str("<span class=\"block\">&nbsp;</span>");
                } else if !trimmed.trim().is_empty() {
                    let line_without_newline = html_line.replace('\n', "");
                    output.push_str(&format!(
                        "<span class=\"block before:content-['$'] before:mr-2 before:text-white/50\">{}</span>",
                        line_without_newline
                    ));
                }
            } else {
                output.push_str(&html_line);
            }
        }

        output
    }
}

impl Default for Highlighter {
    fn default() -> Self {
        Self::new()
    }
}
