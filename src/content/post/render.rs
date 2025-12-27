use crate::content::post::meta::Post;
use maud::{Markup, PreEscaped, html};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};
use std::{collections::HashMap, path::Path};

use super::highlight::highlight_code_block;

#[derive(Debug)]
struct Frame {
    kind: FrameKind,
    buffer: Vec<RenderNode>,
    text: String,
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
        text: String::new(),
    }];
    let mut slug_counts = HashMap::new();
    let mut image_index = 0usize;

    for event in (post.events)() {
        match event {
            Event::Start(tag) => handle_start_event(tag, &mut frames),
            Event::End(tag_end) => handle_end_event(
                tag_end,
                &mut frames,
                &mut slug_counts,
                post.id,
                &mut image_index,
            ),
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
        text: String::new(),
    });
}

fn handle_end_event(
    _tag_end: TagEnd,
    frames: &mut Vec<Frame>,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
) {
    if frames.len() <= 1 {
        return;
    }

    let frame = frames.pop().expect("frame stack underflow");
    let rendered = render_frame(frame, slug_counts, post_id, image_index);
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

fn handle_task_list_marker_event(_checked: bool, frames: &mut [Frame]) {
    append_markup(
        html! {
            input type="checkbox" disabled checked;
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

fn render_nodes(buffer: &[RenderNode]) -> Markup {
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
        @for node in rendered { (node) }
    }
}

fn render_frame(
    frame: Frame,
    slug_counts: &mut HashMap<String, usize>,
    post_id: &str,
    image_index: &mut usize,
) -> RenderNode {
    match frame.kind {
        FrameKind::Root => RenderNode::Markup(render_nodes(&frame.buffer)),
        FrameKind::Paragraph => RenderNode::Markup(html! {
            p class="text-gray-300 mt-4 first:mt-0" { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::Heading(level) => match level {
            HeadingLevel::H1 => render_heading(
                "h1",
                "text-4xl md:text-5xl font-semibold tracking-tight text-white mt-10 mb-6",
                &frame,
                slug_counts,
            ),
            HeadingLevel::H2 => render_heading(
                "h2",
                "text-2xl md:text-3xl font-semibold tracking-tight text-white mt-10 mb-4",
                &frame,
                slug_counts,
            ),
            _ => render_heading(
                "h3",
                "text-xl md:text-2xl font-semibold text-white mt-8 mb-3",
                &frame,
                slug_counts,
            ),
        },
        FrameKind::BlockQuote => RenderNode::Markup(html! {
            blockquote class="text-gray-300" { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::CodeBlock { info, text } => RenderNode::CodeBlock { info, text },
        FrameKind::List(start) => match start {
            Some(start) => RenderNode::Markup(html! {
                ol class="list-decimal pl-6 space-y-2 text-gray-300 mb-4" start=(start) {
                    (render_nodes(&frame.buffer))
                }
            }),
            None => RenderNode::Markup(html! {
                ul class="list-disc pl-6 space-y-2 text-gray-300 mb-4" {
                    (render_nodes(&frame.buffer))
                }
            }),
        },
        FrameKind::Item => RenderNode::Markup(html! {
            li { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::Emphasis => RenderNode::Markup(html! {
            em { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::Strong => RenderNode::Markup(html! {
            strong { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::Strikethrough => RenderNode::Markup(html! {
            del { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::Link { dest_url, title } => {
            let is_external = dest_url.starts_with("http://")
                || dest_url.starts_with("https://")
                || dest_url.starts_with("mailto:");
            if title.is_empty() {
                RenderNode::Markup(if is_external {
                    html! {
                        a href=(dest_url) target="_blank" rel="noopener noreferrer" {
                            (render_nodes(&frame.buffer))
                        }
                    }
                } else {
                    html! {
                        a href=(dest_url) { (render_nodes(&frame.buffer)) }
                    }
                })
            } else {
                RenderNode::Markup(if is_external {
                    html! {
                        a href=(dest_url) title=(title) target="_blank" rel="noopener noreferrer" {
                            (render_nodes(&frame.buffer))
                        }
                    }
                } else {
                    html! {
                        a href=(dest_url) title=(title) { (render_nodes(&frame.buffer)) }
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
            table { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::TableHead => RenderNode::Markup(html! {
            thead { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::TableRow => RenderNode::Markup(html! {
            tr { (render_nodes(&frame.buffer)) }
        }),
        FrameKind::TableCell => RenderNode::Markup(html! {
            td { (render_nodes(&frame.buffer)) }
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
) -> RenderNode {
    let slug = unique_slug(&frame.text, slug_counts);
    let anchor = html! {
        a   class="inline-flex items-center text-white/40 hover:text-white/70 text-base align-middle no-underline border-b-0 opacity-0 group-hover:opacity-100 focus:opacity-100 focus-visible:opacity-100 transition-opacity translate-y-1"
            href={ "#" (slug) }
            aria-label="Link to this section"
        {
            ({
                PreEscaped(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m9 15l6-6m-4-3l.463-.536a5 5 0 0 1 7.071 7.072L18 13m-5 5l-.397.534a5.07 5.07 0 0 1-7.127 0a4.97 4.97 0 0 1 0-7.071L6 11"/></svg>"#,
                )
            })
        }
    };
    let content = render_nodes(&frame.buffer);
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

fn render_code_block(info: &Option<String>, text: &str) -> Markup {
    let language = code_language(info);
    let shell_prompt = matches!(language, Some("sh" | "bash" | "fish"));
    let highlighted = highlight_code_block(text, language, shell_prompt);
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
                ({
                    PreEscaped(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2"/></g></svg>"#,
                    )
                })
            }
            pre class="overflow-x-auto p-4 text-[0.95rem] leading-6 sm:text-sm" {
                code class={ "block font-mono text-gray-100 " (language_class) } {
                    (PreEscaped(highlighted))
                }
            }
        }
    }
}

fn render_code_block_group(run: &[RenderNode]) -> Markup {
    html! {
        div class="mt-3 mb-6 rounded-xl border border-white/10 bg-white/5 shadow-inner overflow-hidden"
        {
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
        div class={ "relative group " (divider_class) } {
            button
                class="code-copy-btn absolute top-3 right-3 text-white/70 hover:text-white border border-white/20 hover:border-white/40 rounded-md p-1.5 transition-colors opacity-0 pointer-events-none group-hover:opacity-100 group-hover:pointer-events-auto"
                type="button"
                aria-label="Copy code"
            {
                ({
                    PreEscaped(
                        r#"<svg xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24"><g fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2"><path d="M9 5H7a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V7a2 2 0 0 0-2-2h-2"/><path d="M9 5a2 2 0 0 1 2-2h2a2 2 0 0 1 2 2a2 2 0 0 1-2 2h-2a2 2 0 0 1-2-2"/></g></svg>"#,
                    )
                })
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
