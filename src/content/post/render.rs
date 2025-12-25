use crate::content::meta::Post;
use maud::{Markup, PreEscaped, html};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};

use super::highlight::highlight_code_block;

#[derive(Debug)]
struct Frame {
    kind: FrameKind,
    buffer: Vec<RenderNode>,
}

#[derive(Debug)]
enum FrameKind {
    Root,
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    CodeBlock {
        info: Option<String>,
        text: String,
    },
    List(Option<u64>),
    Item,
    Emphasis,
    Strong,
    Strikethrough,
    Link {
        dest_url: String,
        title: String,
    },
    Image {
        dest_url: String,
        title: String,
        alt: String,
    },
    Table,
    TableHead,
    TableRow,
    TableCell,
}

#[derive(Debug)]
enum RenderNode {
    Markup(Markup),
    CodeBlock { info: Option<String>, text: String },
}

pub fn render_post(post: &Post) -> Markup {
    let mut frames = vec![Frame {
        kind: FrameKind::Root,
        buffer: Vec::new(),
    }];

    for event in (post.events)() {
        match event {
            Event::Start(tag) => handle_start_event(tag, &mut frames),
            Event::End(tag_end) => handle_end_event(tag_end, &mut frames),
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
    render_nodes(&root.buffer)
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
        Tag::TableCell => FrameKind::TableCell,
        _ => FrameKind::Root,
    };

    frames.push(Frame {
        kind,
        buffer: Vec::new(),
    });
}

fn handle_end_event(_tag_end: TagEnd, frames: &mut Vec<Frame>) {
    if frames.len() <= 1 {
        return;
    }

    let frame = frames.pop().expect("frame stack underflow");
    let rendered = render_frame(frame);
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

    if let Some(Frame {
        kind: FrameKind::Image { alt, .. },
        ..
    }) = frames.last_mut()
    {
        alt.push_str(text.as_ref());
        return;
    }

    append_markup(html! { (text.as_ref()) }, frames);
}

fn handle_code_event(code: CowStr, frames: &mut [Frame]) {
    append_markup(html! { code { (code.as_ref()) } }, frames);
}

fn handle_inline_math_event(text: CowStr, frames: &mut [Frame]) {
    append_markup(html! { span { (text.as_ref()) } }, frames);
}

fn handle_display_math_event(text: CowStr, frames: &mut [Frame]) {
    append_markup(html! { div { (text.as_ref()) } }, frames);
}

fn handle_html_event(raw: CowStr, frames: &mut [Frame]) {
    append_markup(html! { (PreEscaped(raw.as_ref())) }, frames);
}

fn handle_inline_html_event(raw: CowStr, frames: &mut [Frame]) {
    append_markup(html! { (PreEscaped(raw.as_ref())) }, frames);
}

fn handle_footnote_reference_event(label: CowStr, frames: &mut [Frame]) {
    append_markup(html! { sup { (label.as_ref()) } }, frames);
}

fn handle_soft_break_event(frames: &mut [Frame]) {
    append_markup(html! { " " }, frames);
}

fn handle_hard_break_event(frames: &mut [Frame]) {
    append_markup(html! { br; }, frames);
}

fn handle_rule_event(frames: &mut [Frame]) {
    append_markup(html! { hr; }, frames);
}

fn handle_task_list_marker_event(_checked: bool, frames: &mut [Frame]) {
    append_markup(html! { input type="checkbox" disabled? checked?; }, frames);
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

fn render_nodes(buffer: &[RenderNode]) -> Markup {
    let mut rendered = Vec::new();
    let mut idx = 0;
    while idx < buffer.len() {
        match &buffer[idx] {
            RenderNode::Markup(markup) => {
                rendered.push(html! { (markup) });
                idx += 1;
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
                        rendered.push(render_code_block(info, text));
                    }
                } else {
                    rendered.push(render_code_block_group(run));
                }
            }
        }
    }

    html! {
        @for node in rendered {
            (node)
        }
    }
}

fn render_frame(frame: Frame) -> RenderNode {
    match frame.kind {
        FrameKind::Root => RenderNode::Markup(render_nodes(&frame.buffer)),
        FrameKind::Paragraph => RenderNode::Markup(
            html! { p class="text-gray-300" { (render_nodes(&frame.buffer)) } },
        ),
        FrameKind::Heading(level) => match level {
            HeadingLevel::H1 => RenderNode::Markup(html! {
                h1 class="text-4xl md:text-5xl font-semibold tracking-tight text-white mt-10 mb-6" {
                    (render_nodes(&frame.buffer))
                }
            }),
            HeadingLevel::H2 => RenderNode::Markup(html! {
                h2 class="text-2xl md:text-3xl font-semibold tracking-tight text-white mt-10 mb-4" {
                    (render_nodes(&frame.buffer))
                }
            }),
            _ => RenderNode::Markup(html! {
                h3 class="text-xl md:text-2xl font-semibold text-white mt-8 mb-3" {
                    (render_nodes(&frame.buffer))
                }
            }),
        },
        FrameKind::BlockQuote => RenderNode::Markup(
            html! { blockquote class="text-gray-300" { (render_nodes(&frame.buffer)) } },
        ),
        FrameKind::CodeBlock { info, text } => RenderNode::CodeBlock { info, text },
        FrameKind::List(start) => match start {
            Some(start) => RenderNode::Markup(
                html! { ol start=(start) { (render_nodes(&frame.buffer)) } },
            ),
            None => RenderNode::Markup(html! { ul { (render_nodes(&frame.buffer)) } }),
        },
        FrameKind::Item => RenderNode::Markup(
            html! { li class="text-gray-300" { (render_nodes(&frame.buffer)) } },
        ),
        FrameKind::Emphasis => RenderNode::Markup(html! { em { (render_nodes(&frame.buffer)) } }),
        FrameKind::Strong => RenderNode::Markup(html! { strong { (render_nodes(&frame.buffer)) } }),
        FrameKind::Strikethrough => {
            RenderNode::Markup(html! { del { (render_nodes(&frame.buffer)) } })
        }
        FrameKind::Link { dest_url, title } => {
            if title.is_empty() {
                RenderNode::Markup(
                    html! { a href=(dest_url) { (render_nodes(&frame.buffer)) } },
                )
            } else {
                RenderNode::Markup(html! {
                    a href=(dest_url) title=(title) { (render_nodes(&frame.buffer)) }
                })
            }
        }
        FrameKind::Image {
            dest_url,
            title,
            alt,
        } => {
            if title.is_empty() {
                RenderNode::Markup(html! { img src=(dest_url) alt=(alt); })
            } else {
                RenderNode::Markup(html! { img src=(dest_url) alt=(alt) title=(title); })
            }
        }
        FrameKind::Table => RenderNode::Markup(html! { table { (render_nodes(&frame.buffer)) } }),
        FrameKind::TableHead => {
            RenderNode::Markup(html! { thead { (render_nodes(&frame.buffer)) } })
        }
        FrameKind::TableRow => RenderNode::Markup(html! { tr { (render_nodes(&frame.buffer)) } }),
        FrameKind::TableCell => RenderNode::Markup(html! { td { (render_nodes(&frame.buffer)) } }),
    }
}

fn render_code_block(info: &Option<String>, text: &str) -> Markup {
    let language = code_language(info);
    let shell_prompt = matches!(language, Some("sh" | "bash" | "fish"));
    let highlighted = highlight_code_block(text, language, shell_prompt);
    let language_class = language
        .map(|value| format!("language-{}", value))
        .unwrap_or_default();
    html! {
        div class="my-6 rounded-xl border border-white/10 bg-white/5 shadow-inner relative" {
            button class="code-copy-btn absolute top-3 right-3 text-white/70 hover:text-white border border-white/20 hover:border-white/40 rounded-md p-1.5 transition-colors" type="button" aria-label="Copy code" {
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2"/></g></svg>"#))
            }
            pre class="overflow-x-auto p-4 text-sm leading-6" {
                code class={ "block font-mono text-gray-100 " (language_class) } {
                    (PreEscaped(highlighted))
                }
            }
        }
    }
}

fn render_code_block_group(run: &[RenderNode]) -> Markup {
    html! {
        div class="my-6 rounded-xl border border-white/10 bg-white/5 shadow-inner overflow-hidden" {
            @for (idx, node) in run.iter().enumerate() {
                @if let RenderNode::CodeBlock { info, text } = node {
                    (render_code_block_inner(info, text, idx > 0))
                }
            }
        }
    }
}

fn render_code_block_inner(info: &Option<String>, text: &str, has_divider: bool) -> Markup {
    let language = code_language(info);
    let shell_prompt = matches!(language, Some("sh" | "bash" | "fish"));
    let highlighted = highlight_code_block(text, language, shell_prompt);
    let language_class = language
        .map(|value| format!("language-{}", value))
        .unwrap_or_default();
    let divider_class = if has_divider {
        "border-t border-white/10"
    } else {
        ""
    };
    html! {
        div class={ "relative " (divider_class) } {
            button class="code-copy-btn absolute top-3 right-3 text-white/70 hover:text-white border border-white/20 hover:border-white/40 rounded-md p-1.5 transition-colors" type="button" aria-label="Copy code" {
                (PreEscaped(r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2"/></g></svg>"#))
            }
            pre class="overflow-x-auto p-4 text-sm leading-6" {
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
