use syntect::{
    easy::HighlightLines,
    highlighting::Theme,
    html::{IncludeBackground, styled_line_to_highlighted_html},
    parsing::{SyntaxSet, syntax_definition::SyntaxDefinition},
    util::LinesWithEndings,
};

pub struct Highlighter {
    syntax_set: SyntaxSet,
    theme: Theme,
}

impl Highlighter {
    pub fn new() -> Self {
        let mut builder = SyntaxSet::load_defaults_newlines().into_builder();
        for source in [
            include_str!("../../../syntaxes/llvm.sublime-syntax"),
            include_str!("../../../syntaxes/aarch64.sublime-syntax"),
            include_str!("../../../syntaxes/toml.sublime-syntax"),
        ] {
            let syntax = SyntaxDefinition::load_from_str(source, true, None)
                .expect("vendored sublime-syntax should parse");
            builder.add(syntax);
        }
        let syntax_set = builder.build();
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
        let mut continuation = false;

        while let Some(line) = lines.next() {
            let ranges = highlighter
                .highlight_line(line, &self.syntax_set)
                .unwrap_or_default();
            let html_line = styled_line_to_highlighted_html(&ranges, IncludeBackground::No)
                .unwrap_or_else(|_| line.to_string());

            if shell_prompt {
                let trimmed = line.trim_end_matches('\n');
                let is_trailing_empty = trimmed.trim().is_empty() && lines.peek().is_none();
                if trimmed.trim().is_empty() {
                    if !is_trailing_empty {
                        output.push_str("<span class=\"block\">&nbsp;</span>");
                    }
                    continuation = false;
                } else {
                    let line_without_newline = html_line.replace('\n', "");
                    if continuation {
                        output.push_str(&format!(
                            "<span class=\"block\">{}</span>",
                            line_without_newline
                        ));
                    } else {
                        output.push_str(&format!(
                            "<span class=\"block before:content-['$'] before:mr-2 before:text-white/50\">{}</span>",
                            line_without_newline
                        ));
                    }
                    let end = trimmed.trim_end();
                    continuation = end.ends_with('\\')
                        || end.ends_with("&&")
                        || end.ends_with("||")
                        || end.ends_with('|');
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn distinct_colors(html: &str) -> HashSet<&str> {
        html.match_indices("style=\"color:#")
            .map(|(i, _)| &html[i + 14..i + 20])
            .collect()
    }

    #[test]
    fn highlights_llvm_blocks_with_multiple_colors() {
        let code = "; comment\ndefine i64 @known(i64 %x) {\nstart:\n  %r = add i64 %x, 1\n  ret i64 %r\n}\n";
        let html = Highlighter::new().highlight_code_block(code, Some("llvm"), false);
        let colors = distinct_colors(&html);
        assert!(
            colors.len() >= 4,
            "expected at least 4 colors, got {colors:?}"
        );
        assert!(
            colors.contains("65737e"),
            "comment gray missing: {colors:?}"
        );
    }

    #[test]
    fn highlights_asm_blocks_with_multiple_colors() {
        let code = "__ZN6devirt7unknown17h..E:\n    ldr x3, [x1, #24]  ; the apply slot\n    ret\n";
        let html = Highlighter::new().highlight_code_block(code, Some("asm"), false);
        let colors = distinct_colors(&html);
        assert!(
            colors.len() >= 3,
            "expected at least 3 colors, got {colors:?}"
        );
    }

    #[test]
    fn highlights_toml_blocks_with_multiple_colors() {
        let code = "[profile.release]\nopt-level = 3\nlto = \"fat\"\ncodegen-units = 1\n";
        let html = Highlighter::new().highlight_code_block(code, Some("toml"), false);
        let colors = distinct_colors(&html);
        assert!(
            colors.len() >= 3,
            "expected at least 3 colors, got {colors:?}"
        );
    }

    #[test]
    fn shell_prompt_skips_continuation_lines() {
        let code = "rustup component add llvm-tools-preview \\\n  && cargo build --release \\\n     --target \"$TARGET\"\necho done\n";
        let html = Highlighter::new().highlight_code_block(code, Some("bash"), true);
        let prompts = html.matches("before:content-['$']").count();
        assert_eq!(
            prompts, 2,
            "only command-starting lines get a prompt: {html}"
        );
    }

    #[test]
    fn shell_prompt_resumes_after_operator_and_blank_line() {
        let code = "cargo build &&\ncargo test\n\ncargo run\n";
        let html = Highlighter::new().highlight_code_block(code, Some("bash"), true);
        let prompts = html.matches("before:content-['$']").count();
        assert_eq!(prompts, 2, "`&&` continues, blank line resets: {html}");
    }

    #[test]
    fn unknown_language_falls_back_to_plain_text() {
        let html = Highlighter::new().highlight_code_block("hello world\n", Some("qqqq"), false);
        let colors = distinct_colors(&html);
        assert_eq!(
            colors.len(),
            1,
            "plain text should be one color: {colors:?}"
        );
        assert!(colors.contains("c0c5ce"));
    }
}
