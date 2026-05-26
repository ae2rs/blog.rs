use maud::{Markup, PreEscaped, html};
use pulldown_cmark::{
    CodeBlockKind, CowStr, Event, HeadingLevel, Options, Parser, Tag, TextMergeStream,
};
use std::{collections::HashMap, path::Path};

use super::types::{Frame, FrameKind, Post, RenderNode};
use crate::{component::icons, content::format::highlight::Highlighter};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CalloutKind {
    Note,
    Warning,
    Danger,
}

impl CalloutKind {
    fn from_code_block(info: &Option<String>) -> Option<Self> {
        match code_language(info) {
            Some(language) if language.eq_ignore_ascii_case("note") => Some(Self::Note),
            Some(language) if language.eq_ignore_ascii_case("warning") => Some(Self::Warning),
            Some(language) if language.eq_ignore_ascii_case("danger") => Some(Self::Danger),
            _ => None,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Note => "Note",
            Self::Warning => "Warning",
            Self::Danger => "Danger",
        }
    }

    fn icon(self) -> &'static str {
        match self {
            Self::Note => icons::NOTE,
            Self::Warning => icons::WARNING,
            Self::Danger => icons::DANGER,
        }
    }

    fn panel_classes(self) -> &'static str {
        match self {
            Self::Note => {
                "my-6 rounded-2xl border border-sky-400/30 bg-sky-400/10 px-4 py-4 shadow-sm shadow-sky-950/20"
            }
            Self::Warning => {
                "my-6 rounded-2xl border border-amber-400/35 bg-amber-300/10 px-4 py-4 shadow-sm shadow-amber-950/20"
            }
            Self::Danger => {
                "my-6 rounded-2xl border border-red-400/35 bg-red-400/10 px-4 py-4 shadow-sm shadow-red-950/20"
            }
        }
    }

    fn icon_classes(self) -> &'static str {
        match self {
            Self::Note => {
                "flex size-8 shrink-0 items-center justify-center rounded-full bg-sky-300/15 text-sky-100"
            }
            Self::Warning => {
                "flex size-8 shrink-0 items-center justify-center rounded-full bg-amber-300/20 text-amber-100"
            }
            Self::Danger => {
                "flex size-8 shrink-0 items-center justify-center rounded-full bg-red-300/20 text-red-100"
            }
        }
    }

    fn title_classes(self) -> &'static str {
        match self {
            Self::Note => "text-sm font-semibold tracking-[0.08em] uppercase text-sky-100",
            Self::Warning => "text-sm font-semibold tracking-[0.08em] uppercase text-amber-100",
            Self::Danger => "text-sm font-semibold tracking-[0.08em] uppercase text-red-100",
        }
    }
}

pub fn render_post(post: &Post, highlighter: &Highlighter) -> Markup {
    let mut slug_counts = HashMap::new();
    let mut image_index = 0usize;

    render_markdown(
        (post.events)(),
        &mut slug_counts,
        post.id,
        &mut image_index,
        highlighter,
    )
}

fn render_markdown<'a, I>(
    events: I,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
    highlighter: &Highlighter,
) -> Markup
where
    I: IntoIterator<Item = Event<'a>>,
{
    let mut frames = vec![Frame {
        kind: FrameKind::Root,
        buffer: Vec::new(),
        text: String::new(),
    }];

    for event in events {
        match event {
            Event::Start(tag) => handle_start_event(tag, &mut frames),
            Event::End(_) => {
                handle_end_event(&mut frames, slug_counts, post_id, image_index, highlighter)
            }
            Event::Text(text) => handle_text_event(text, &mut frames),
            Event::Code(code) => handle_code_event(code, &mut frames),
            Event::InlineMath(text) => handle_inline_math_event(text, &mut frames),
            Event::DisplayMath(text) => handle_display_math_event(text, &mut frames),
            Event::Html(raw) => handle_html_event(raw, &mut frames),
            Event::InlineHtml(raw) => handle_inline_html_event(raw, &mut frames),
            Event::FootnoteReference(label) => handle_footnote_reference_event(label, &mut frames),
            Event::SoftBreak => handle_soft_break_event(&mut frames),
            Event::HardBreak => handle_hard_break_event(&mut frames),
            Event::Rule => handle_rule_event(&mut frames),
            Event::TaskListMarker(checked) => handle_task_list_marker_event(checked, &mut frames),
        }
    }

    let root = frames
        .pop()
        .expect("render_post should always keep a root frame");
    render_nodes(&root.buffer, highlighter)
}

fn render_markdown_fragment(
    markdown: &str,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
    highlighter: &Highlighter,
) -> Markup {
    render_markdown(
        TextMergeStream::new(Parser::new_ext(markdown, Options::ENABLE_TABLES)),
        slug_counts,
        post_id,
        image_index,
        highlighter,
    )
}

fn handle_start_event(tag: Tag, frames: &mut Vec<Frame>) {
    let kind = match tag {
        Tag::Paragraph => FrameKind::Paragraph,
        Tag::Heading { level, .. } => FrameKind::Heading(level),
        Tag::BlockQuote(_) => FrameKind::BlockQuote,
        Tag::CodeBlock(kind) => FrameKind::CodeBlock {
            info: match kind {
                CodeBlockKind::Fenced(info) => Some(info.to_string()),
                CodeBlockKind::Indented => None,
            },
            text: String::new(),
        },
        Tag::List(start) => FrameKind::List(start),
        Tag::Item => FrameKind::Item,
        Tag::Emphasis => FrameKind::Emphasis,
        Tag::Strong => FrameKind::Strong,
        Tag::Strikethrough => FrameKind::Strikethrough,
        Tag::Link {
            dest_url, title, ..
        } => FrameKind::Link {
            dest_url: dest_url.to_string(),
            title: title.to_string(),
        },
        Tag::Image {
            dest_url, title, ..
        } => FrameKind::Image {
            dest_url: dest_url.to_string(),
            title: title.to_string(),
            alt: String::new(),
        },
        Tag::Table(_) => FrameKind::Table,
        Tag::TableHead => FrameKind::TableHead,
        Tag::TableRow => FrameKind::TableRow,
        Tag::TableCell => {
            let in_head = frames
                .iter()
                .rev()
                .any(|f| matches!(f.kind, FrameKind::TableHead));
            if in_head {
                FrameKind::TableHeadCell
            } else {
                FrameKind::TableCell
            }
        }
        _ => FrameKind::Root,
    };

    frames.push(Frame {
        kind,
        buffer: Vec::new(),
        text: String::new(),
    });
}

fn handle_end_event(
    frames: &mut Vec<Frame>,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
    highlighter: &Highlighter,
) {
    if frames.len() <= 1 {
        return;
    }

    let frame = frames.pop().expect("frame stack underflow");
    let rendered = render_frame(frame, slug_counts, post_id, image_index, highlighter);
    append_node(rendered, frames);
}

fn handle_text_event(text: CowStr, frames: &mut [Frame]) {
    if let Some(Frame {
        kind: FrameKind::CodeBlock { text: buffer, .. },
        ..
    }) = frames.last_mut()
    {
        buffer.push_str(text.as_ref());
        return;
    }

    append_heading_text(frames, text.as_ref());

    if let Some(Frame {
        kind: FrameKind::Image { alt, .. },
        ..
    }) = frames.last_mut()
    {
        alt.push_str(text.as_ref());
        return;
    }

    append_markup(
        html! {
            (text.as_ref())
        },
        frames,
    );
}

fn handle_code_event(code: CowStr, frames: &mut [Frame]) {
    append_heading_text(frames, code.as_ref());
    append_markup(
        html! {
            code
                class="text-[0.95em] bg-white/10 px-1 py-0.5 rounded box-decoration-clone [box-decoration-break:clone]"
            { (code.as_ref()) }
        },
        frames,
    );
}

fn handle_inline_math_event(text: CowStr, frames: &mut [Frame]) {
    append_markup(
        html! {
            span { (text.as_ref()) }
        },
        frames,
    );
}

fn handle_display_math_event(text: CowStr, frames: &mut [Frame]) {
    append_markup(
        html! {
            div { (text.as_ref()) }
        },
        frames,
    );
}

fn handle_html_event(raw: CowStr, frames: &mut [Frame]) {
    append_markup(
        html! {
            (PreEscaped(raw.as_ref()))
        },
        frames,
    );
}

fn handle_inline_html_event(raw: CowStr, frames: &mut [Frame]) {
    append_markup(
        html! {
            (PreEscaped(raw.as_ref()))
        },
        frames,
    );
}

fn handle_footnote_reference_event(label: CowStr, frames: &mut [Frame]) {
    append_markup(
        html! {
            sup { (label.as_ref()) }
        },
        frames,
    );
}

fn handle_soft_break_event(frames: &mut [Frame]) {
    append_markup(
        html! {
            " "
        },
        frames,
    );
}

fn handle_hard_break_event(frames: &mut [Frame]) {
    append_markup(
        html! {
            br;
        },
        frames,
    );
}

fn handle_rule_event(frames: &mut [Frame]) {
    append_markup(
        html! {
            hr;
        },
        frames,
    );
}

fn handle_task_list_marker_event(checked: bool, frames: &mut [Frame]) {
    append_markup(
        html! {
            @if checked {
                input type="checkbox" disabled checked;
            } @else {
                input type="checkbox" disabled;
            }
        },
        frames,
    );
}

fn append_markup(markup: Markup, frames: &mut [Frame]) {
    if let Some(frame) = frames.last_mut() {
        frame.buffer.push(RenderNode::Markup(markup));
    }
}

fn append_node(node: RenderNode, frames: &mut [Frame]) {
    if let Some(frame) = frames.last_mut() {
        frame.buffer.push(node);
    }
}

fn render_nodes(buffer: &[RenderNode], highlighter: &Highlighter) -> Markup {
    let mut rendered = Vec::new();
    let mut idx = 0;
    while idx < buffer.len() {
        match &buffer[idx] {
            RenderNode::Markup(markup) => {
                rendered.push(html! {
                    (markup)
                });
                idx += 1;
            }
            RenderNode::Paragraph { content } => {
                rendered.push(render_paragraph(content));
                idx += 1;
            }
            RenderNode::BlockQuote { .. } => {
                let start = idx;
                while idx < buffer.len() {
                    if matches!(buffer[idx], RenderNode::BlockQuote { .. }) {
                        idx += 1;
                    } else {
                        break;
                    }
                }
                rendered.push(render_blockquote_group(&buffer[start..idx], highlighter));
            }
            RenderNode::CodeBlock { .. } => {
                let start = idx;
                while idx < buffer.len() {
                    if matches!(buffer[idx], RenderNode::CodeBlock { .. }) {
                        idx += 1;
                    } else {
                        break;
                    }
                }
                let run = &buffer[start..idx];
                if run.len() == 1 {
                    if let RenderNode::CodeBlock { info, text } = &run[0] {
                        rendered.push(render_code_block(info, text, highlighter));
                    }
                } else {
                    rendered.push(render_code_block_group(run, highlighter));
                }
            }
        }
    }

    html! {
        @for node in rendered { (node) }
    }
}

fn render_frame(
    frame: Frame,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
    highlighter: &Highlighter,
) -> RenderNode {
    match frame.kind {
        FrameKind::Root => RenderNode::Markup(render_nodes(&frame.buffer, highlighter)),
        FrameKind::Paragraph => RenderNode::Paragraph {
            content: render_nodes(&frame.buffer, highlighter).into_string(),
        },
        FrameKind::Heading(level) => match level {
            HeadingLevel::H1 => render_heading(
                "h1",
                "text-4xl md:text-5xl font-semibold tracking-tight text-white mt-10 mb-6",
                &frame,
                slug_counts,
                highlighter,
            ),
            HeadingLevel::H2 => render_heading(
                "h2",
                "text-2xl md:text-3xl font-semibold tracking-tight text-white mt-10 mb-4",
                &frame,
                slug_counts,
                highlighter,
            ),
            _ => render_heading(
                "h3",
                "text-xl md:text-2xl font-semibold text-white mt-8 mb-3",
                &frame,
                slug_counts,
                highlighter,
            ),
        },
        FrameKind::BlockQuote => RenderNode::BlockQuote {
            buffer: frame.buffer,
        },
        FrameKind::CodeBlock { info, text } => {
            if let Some(kind) = CalloutKind::from_code_block(&info) {
                RenderNode::Markup(render_callout(
                    kind,
                    &text,
                    slug_counts,
                    post_id,
                    image_index,
                    highlighter,
                ))
            } else {
                RenderNode::CodeBlock { info, text }
            }
        }
        FrameKind::List(start) => match start {
            Some(start) => RenderNode::Markup(html! {
                ol class="list-decimal pl-6 space-y-2 text-gray-300 mb-4" start=(start) {
                    (render_nodes(&frame.buffer, highlighter))
                }
            }),
            None => RenderNode::Markup(html! {
                ul class="list-disc pl-6 space-y-2 text-gray-300 mb-4" {
                    (render_nodes(&frame.buffer, highlighter))
                }
            }),
        },
        FrameKind::Item => RenderNode::Markup(html! {
            li { (render_nodes(&frame.buffer, highlighter)) }
        }),
        FrameKind::Emphasis => RenderNode::Markup(html! {
            em { (render_nodes(&frame.buffer, highlighter)) }
        }),
        FrameKind::Strong => RenderNode::Markup(html! {
            strong { (render_nodes(&frame.buffer, highlighter)) }
        }),
        FrameKind::Strikethrough => RenderNode::Markup(html! {
            del { (render_nodes(&frame.buffer, highlighter)) }
        }),
        FrameKind::Link { dest_url, title } => {
            let is_external = dest_url.starts_with("http://")
                || dest_url.starts_with("https://")
                || dest_url.starts_with("mailto:");
            if title.is_empty() {
                RenderNode::Markup(if is_external {
                    html! {
                        a href=(dest_url) target="_blank" rel="noopener noreferrer" {
                            (render_nodes(&frame.buffer, highlighter))
                        }
                    }
                } else {
                    html! {
                        a href=(dest_url) { (render_nodes(&frame.buffer, highlighter)) }
                    }
                })
            } else {
                RenderNode::Markup(if is_external {
                    html! {
                        a href=(dest_url) title=(title) target="_blank" rel="noopener noreferrer" {
                            (render_nodes(&frame.buffer, highlighter))
                        }
                    }
                } else {
                    html! {
                        a href=(dest_url) title=(title) { (render_nodes(&frame.buffer, highlighter)) }
                    }
                })
            }
        }
        FrameKind::Image {
            dest_url,
            title,
            alt,
        } => {
            let dest_url = resolve_image_src(&dest_url, post_id, image_index);
            RenderNode::Markup(html! {
                figure class="flex flex-col items-center my-6" {
                    @if title.is_empty() {
                        img class="max-w-full rounded-md border border-white/10"
                            src=(dest_url)
                            alt=(alt);
                    } @else {
                        img class="max-w-full rounded-md border border-white/10"
                            src=(dest_url)
                            alt=(alt)
                            title=(title);
                        figcaption class="mt-2 text-sm text-gray-400 text-center" { (title) }
                    }
                }
            })
        }
        FrameKind::Table => RenderNode::Markup(html! {
            div class="my-6 w-full overflow-x-auto rounded-xl border border-white/10" {
                table class="w-full border-collapse text-sm" {
                    (render_nodes(&frame.buffer, highlighter))
                }
            }
        }),
        FrameKind::TableHead => RenderNode::Markup(html! {
            thead class="bg-white/5" {
                (render_nodes(&frame.buffer, highlighter))
            }
        }),
        FrameKind::TableRow => RenderNode::Markup(html! {
            tr class="even:bg-white/[0.03]" {
                (render_nodes(&frame.buffer, highlighter))
            }
        }),
        FrameKind::TableHeadCell => RenderNode::Markup(html! {
            th class="px-4 py-3.5 text-left font-semibold text-white border-b border-white/15" {
                (render_nodes(&frame.buffer, highlighter))
            }
        }),
        FrameKind::TableCell => RenderNode::Markup(html! {
            td class="px-4 py-3.5 text-gray-300 border-b border-white/[0.08] [tr:last-child_&]:border-0" {
                (render_nodes(&frame.buffer, highlighter))
            }
        }),
    }
}

fn append_heading_text(frames: &mut [Frame], text: &str) {
    if let Some(frame) = frames
        .iter_mut()
        .rev()
        .find(|frame| matches!(frame.kind, FrameKind::Heading(_)))
    {
        frame.text.push_str(text);
    }
}

fn render_heading(
    tag: &str,
    classes: &str,
    frame: &Frame,
    slug_counts: &mut HashMap<String, usize>,
    highlighter: &Highlighter,
) -> RenderNode {
    let slug = unique_slug(&frame.text, slug_counts);
    let anchor = html! {
        a   class="inline-flex items-center text-white/40 hover:text-white/70 text-base align-middle no-underline border-b-0 opacity-0 group-hover:opacity-100 focus:opacity-100 focus-visible:opacity-100 transition-opacity translate-y-1"
            href={ "#" (slug) }
            aria-label="Link to this section"
        {
            (PreEscaped(icons::LINK))
        }
    };
    let content = render_nodes(&frame.buffer, highlighter);
    let heading_classes = format!("{} group flex items-baseline gap-3", classes);
    RenderNode::Markup(match tag {
        "h1" => html! {
            h1 id=(slug) class=(heading_classes) {
                span class="min-w-0" { (content) }
                (anchor)
            }
        },
        "h2" => html! {
            h2 id=(slug) class=(heading_classes) {
                span class="min-w-0" { (content) }
                (anchor)
            }
        },
        _ => html! {
            h3 id=(slug) class=(heading_classes) {
                span class="min-w-0" { (content) }
                (anchor)
            }
        },
    })
}

fn unique_slug(text: &str, slug_counts: &mut HashMap<String, usize>) -> String {
    let base = slugify(text);
    let base = if base.is_empty() {
        "section".to_string()
    } else {
        base
    };
    let entry = slug_counts.entry(base.clone()).or_insert(0);
    *entry += 1;
    if *entry == 1 {
        base
    } else {
        format!("{}-{}", base, entry)
    }
}

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = false;
    for ch in text.to_lowercase().chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn resolve_image_src(dest_url: &str, post_id: &str, image_index: &mut usize) -> String {
    if !is_local_image(dest_url) {
        return dest_url.to_string();
    }
    let extension = Path::new(dest_url)
        .extension()
        .and_then(|ext| ext.to_str())
        .expect("local image URLs must include a file extension");
    *image_index += 1;
    format!("/img/{}/{}.{}", post_id, *image_index, extension)
}

fn is_local_image(dest_url: &str) -> bool {
    if dest_url.starts_with('/') {
        return false;
    }
    if dest_url.starts_with("http://")
        || dest_url.starts_with("https://")
        || dest_url.starts_with("mailto:")
        || dest_url.starts_with("data:")
    {
        return false;
    }
    !dest_url.contains("://")
}

fn render_callout(
    kind: CalloutKind,
    text: &str,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
    highlighter: &Highlighter,
) -> Markup {
    let content = render_markdown_fragment(text, slug_counts, post_id, image_index, highlighter);

    html! {
        aside class=(kind.panel_classes()) {
            div class="flex items-center gap-3" {
                span class=(kind.icon_classes()) {
                    (PreEscaped(kind.icon()))
                }
                span class=(kind.title_classes()) {
                    (kind.label())
                }
            }
            div class="mt-3 text-sm leading-7 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_p]:text-gray-100 [&_ul]:text-gray-100 [&_ol]:text-gray-100 [&_li]:text-gray-100 [&_blockquote]:my-4 [&_a:hover]:text-white [&_code]:bg-black/20 [&_code]:text-white" {
                (content)
            }
        }
    }
}

fn render_paragraph(content: &str) -> Markup {
    html! {
        p class="text-gray-300 mt-4 first:mt-0" {
            (PreEscaped(content))
        }
    }
}

fn render_blockquote_group(run: &[RenderNode], highlighter: &Highlighter) -> Markup {
    let mut content_nodes = Vec::new();
    for node in run {
        if let RenderNode::BlockQuote { buffer } = node {
            for child in buffer {
                content_nodes.push(child);
            }
        }
    }

    let footer_html = content_nodes.last().and_then(|node| match node {
        RenderNode::Paragraph { content } => strip_quote_footer_prefix(content),
        _ => None,
    });

    html! {
        blockquote class="my-6 border-l-2 border-white/15 pl-5 text-[1.03rem] leading-8 [&>*:first-child]:mt-0 [&>*:last-child]:mb-0 [&_p]:italic [&_p]:text-gray-200/90 [&_p]:leading-8 [&_p]:tracking-[0.01em] [&_strong]:text-white [&_ul]:text-gray-200/90 [&_ol]:text-gray-200/90 [&_li]:italic [&_li]:text-gray-200/90 [&_p+ul]:mt-4 [&_p+ol]:mt-4" {
            @for node in content_nodes.iter().take(content_nodes.len().saturating_sub(usize::from(footer_html.is_some()))) {
                @match node {
                    RenderNode::Markup(markup) => { (markup) }
                    RenderNode::Paragraph { content } => { (render_paragraph(content)) }
                    RenderNode::BlockQuote { buffer } => { (render_nodes(buffer, highlighter)) }
                    RenderNode::CodeBlock { info, text } => { (render_code_block(info, text, highlighter)) }
                }
            }
            @if let Some(footer_html) = footer_html {
                footer class="mt-4 text-sm not-italic tracking-[0.03em] text-gray-400 [&_a]:text-gray-300 [&_a:hover]:text-white [&_strong]:text-gray-200" {
                    (PreEscaped(footer_html))
                }
            }
        }
    }
}

fn strip_quote_footer_prefix(content: &str) -> Option<&str> {
    content
        .strip_prefix("-- ")
        .or_else(|| content.strip_prefix("— "))
}

fn render_code_block(info: &Option<String>, text: &str, highlighter: &Highlighter) -> Markup {
    let language = code_language(info);
    let shell_prompt = matches!(language, Some("sh" | "bash" | "fish"));
    let highlighted = highlighter.highlight_code_block(text, language, shell_prompt);
    let language_class = language
        .map(|value| format!("language-{}", value))
        .unwrap_or_default();
    html! {
        div class="mt-3 mb-6 rounded-xl border border-white/10 bg-white/5 shadow-inner relative group"
        {
            button
                class="code-copy-btn absolute top-3 right-3 text-white/70 hover:text-white border border-white/20 hover:border-white/40 rounded-md p-1.5 transition-colors opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto"
                type="button"
                aria-label="Copy code"
            {
                (PreEscaped(icons::CODE_COPY))
            }
            pre class="overflow-x-auto p-4 text-[0.95rem] leading-6 sm:text-sm" {
                code class={ "block font-mono text-gray-100 " (language_class) } {
                    (PreEscaped(highlighted))
                }
            }
        }
    }
}

fn render_code_block_group(run: &[RenderNode], highlighter: &Highlighter) -> Markup {
    html! {
        div class="mt-3 mb-6 rounded-xl border border-white/10 bg-white/5 shadow-inner overflow-hidden"
        {
            @for (idx, node) in run.iter().enumerate() {
                @if let RenderNode::CodeBlock { info, text } = node {
                    (render_code_block_inner(info, text, idx > 0, highlighter))
                }
            }
        }
    }
}

fn render_code_block_inner(
    info: &Option<String>,
    text: &str,
    has_divider: bool,
    highlighter: &Highlighter,
) -> Markup {
    let language = code_language(info);
    let shell_prompt = matches!(language, Some("sh" | "bash" | "fish"));
    let highlighted = highlighter.highlight_code_block(text, language, shell_prompt);
    let language_class = language
        .map(|value| format!("language-{}", value))
        .unwrap_or_default();
    let divider_class = if has_divider {
        "border-t border-white/10"
    } else {
        ""
    };
    html! {
        div class={ "relative group " (divider_class) } {
            button
                class="code-copy-btn absolute top-3 right-3 text-white/70 hover:text-white border border-white/20 hover:border-white/40 rounded-md p-1.5 transition-colors opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto"
                type="button"
                aria-label="Copy code"
            {
                (PreEscaped(icons::CODE_COPY))
            }
            pre class="overflow-x-auto p-4 text-[0.95rem] leading-6 sm:text-sm" {
                code class={ "block font-mono text-gray-100 " (language_class) } {
                    (PreEscaped(highlighted))
                }
            }
        }
    }
}

fn code_language(info: &Option<String>) -> Option<&str> {
    info.as_deref()
        .and_then(|value| value.split_whitespace().next())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(markdown: &str) -> String {
        let highlighter = Highlighter::default();
        let mut slug_counts = HashMap::new();
        let mut image_index = 0usize;

        render_markdown_fragment(
            markdown,
            &mut slug_counts,
            "test-post",
            &mut image_index,
            &highlighter,
        )
        .into_string()
    }

    fn count_matches(haystack: &str, needle: &str) -> usize {
        haystack.matches(needle).count()
    }

    #[test]
    fn renders_styled_blockquotes() {
        let html = render("> Quoted line");

        assert!(html.contains("<blockquote"));
        assert!(html.contains("border-l-2 border-white/15"));
        assert!(html.contains("Quoted line"));
    }

    #[test]
    fn merges_adjacent_blockquote_segments_for_lists() {
        let html = render("> Quoted intro\n> Heading\n\n> - one\n> - two");

        assert_eq!(count_matches(&html, "<blockquote"), 1);
        assert!(html.contains("Quoted intro"));
        assert!(html.contains("Heading"));
        assert!(html.contains("<ul class=\"list-disc"));
    }

    #[test]
    fn renders_quote_footer_from_final_paragraph() {
        let html = render("> Quoted line\n>\n> -- Burton Bloom");

        assert_eq!(count_matches(&html, "<blockquote"), 1);
        assert!(html.contains("Quoted line"));
        assert!(html.contains("<footer"));
        assert!(html.contains("Burton Bloom"));
        assert!(!html.contains("-- Burton Bloom</p>"));
    }

    #[test]
    fn renders_quote_footer_with_inline_markdown() {
        let html = render("> Quoted line\n>\n> -- [Burton Bloom](/authors/burton)");

        assert!(html.contains("<footer"));
        assert!(html.contains("href=\"/authors/burton\""));
        assert!(!html.contains("-- <a"));
    }

    #[test]
    fn renders_note_callouts_with_markdown_content() {
        let html =
            render("```note\nA *small* [note](/note) with `inline` code.\n\n- one\n- two\n```");

        assert!(html.contains("<aside"));
        assert!(html.contains("border-sky-400/30"));
        assert!(html.contains(">Note<"));
        assert!(html.contains("<em>small</em>"));
        assert!(html.contains("href=\"/note\""));
        assert!(html.contains("code class=\"text-[0.95em] bg-white/10"));
        assert!(html.contains("<ul class=\"list-disc"));
        assert_eq!(count_matches(&html, "code-copy-btn"), 0);
    }

    #[test]
    fn renders_warning_and_danger_callouts() {
        let html = render("```warning\nHeads up.\n```\n\n```danger\nStop now.\n```");

        assert!(html.contains("border-amber-400/35"));
        assert!(html.contains(">Warning<"));
        assert!(html.contains("border-red-400/35"));
        assert!(html.contains(">Danger<"));
    }

    #[test]
    fn renders_styled_tables() {
        let html = render("| A | B |\n|---|---|\n| 1 | 2 |");
        assert!(html.contains("<table"));
        assert!(html.contains("<th "));
        assert!(html.contains("<td "));
        assert!(html.contains("border-white/10"));
        assert!(html.contains("A"));
        assert!(html.contains("1"));
    }

    #[test]
    fn keeps_regular_code_blocks_unchanged() {
        let html = render("```rust\nfn main() {}\n```");

        assert!(html.contains("code-copy-btn"));
        assert!(html.contains("language-rust"));
        assert!(!html.contains("<aside"));
    }

    #[test]
    fn does_not_group_notes_with_code_blocks() {
        let html = render(
            "```rust\nfn before() {}\n```\n\n```note\nBetween\n```\n\n```bash\necho hi\n```",
        );

        assert!(html.contains("language-rust"));
        assert!(html.contains("language-bash"));
        assert!(html.contains(">Note<"));
        assert_eq!(count_matches(&html, "code-copy-btn"), 2);
        assert_eq!(count_matches(&html, "<aside"), 1);
    }
}
