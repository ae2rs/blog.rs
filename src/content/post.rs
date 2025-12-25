use crate::{common::layout, content::meta::Post, pages};
use axum::{extract::Path, http::StatusCode};
use macros::Post;
use maud::{Markup, PreEscaped, html};
use pulldown_cmark::{CodeBlockKind, CowStr, Event, HeadingLevel, Tag, TagEnd};

#[derive(Post)]
struct Posts;

fn get_post_by_id(id: &str) -> Option<&'static Post> {
    Posts::get_published(id)
}

#[derive(Debug)]
struct Frame {
    kind: FrameKind,
    buffer: Vec<Markup>,
}

#[derive(Debug)]
enum FrameKind {
    Root,
    Paragraph,
    Heading(HeadingLevel),
    BlockQuote,
    CodeBlock(Option<String>),
    List(Option<u64>),
    Item,
    Emphasis,
    Strong,
    Strikethrough,
    Link { dest_url: String, title: String },
    Image { dest_url: String, title: String, alt: String },
    Table,
    TableHead,
    TableRow,
    TableCell,
}

fn render_post(post: &Post) -> Markup {
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
            Event::FootnoteReference(label) => {
                handle_footnote_reference_event(label, &mut frames)
            }
            Event::SoftBreak => handle_soft_break_event(&mut frames),
            Event::HardBreak => handle_hard_break_event(&mut frames),
            Event::Rule => handle_rule_event(&mut frames),
            Event::TaskListMarker(checked) => {
                handle_task_list_marker_event(checked, &mut frames)
            }
        }
    }

    let root = frames
        .pop()
        .expect("render_post should always keep a root frame");
    render_buffer(&root.buffer)
}

fn handle_start_event(tag: Tag, frames: &mut Vec<Frame>) {
    let kind = match tag {
        Tag::Paragraph => FrameKind::Paragraph,
        Tag::Heading { level, .. } => FrameKind::Heading(level),
        Tag::BlockQuote(_) => FrameKind::BlockQuote,
        Tag::CodeBlock(kind) => FrameKind::CodeBlock(match kind {
            CodeBlockKind::Fenced(info) => Some(info.to_string()),
            CodeBlockKind::Indented => None,
        }),
        Tag::List(start) => FrameKind::List(start),
        Tag::Item => FrameKind::Item,
        Tag::Emphasis => FrameKind::Emphasis,
        Tag::Strong => FrameKind::Strong,
        Tag::Strikethrough => FrameKind::Strikethrough,
        Tag::Link { dest_url, title, .. } => FrameKind::Link {
            dest_url: dest_url.to_string(),
            title: title.to_string(),
        },
        Tag::Image { dest_url, title, .. } => FrameKind::Image {
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
    append_markup(rendered, frames);
}

fn handle_text_event(text: CowStr, frames: &mut Vec<Frame>) {
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

fn handle_code_event(code: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { code { (code.as_ref()) } }, frames);
}

fn handle_inline_math_event(text: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { span { (text.as_ref()) } }, frames);
}

fn handle_display_math_event(text: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { div { (text.as_ref()) } }, frames);
}

fn handle_html_event(raw: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { (PreEscaped(raw.as_ref())) }, frames);
}

fn handle_inline_html_event(raw: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { (PreEscaped(raw.as_ref())) }, frames);
}

fn handle_footnote_reference_event(label: CowStr, frames: &mut Vec<Frame>) {
    append_markup(html! { sup { (label.as_ref()) } }, frames);
}

fn handle_soft_break_event(frames: &mut Vec<Frame>) {
    append_markup(html! { " " }, frames);
}

fn handle_hard_break_event(frames: &mut Vec<Frame>) {
    append_markup(html! { br; }, frames);
}

fn handle_rule_event(frames: &mut Vec<Frame>) {
    append_markup(html! { hr; }, frames);
}

fn handle_task_list_marker_event(_checked: bool, frames: &mut Vec<Frame>) {
    append_markup(
        html! { input type="checkbox" disabled? checked?; },
        frames,
    );
}

fn append_markup(markup: Markup, frames: &mut Vec<Frame>) {
    if let Some(frame) = frames.last_mut() {
        frame.buffer.push(markup);
    }
}

fn render_buffer(buffer: &[Markup]) -> Markup {
    html! {
        @for node in buffer {
            (node)
        }
    }
}

fn render_frame(frame: Frame) -> Markup {
    match frame.kind {
        FrameKind::Root => render_buffer(&frame.buffer),
        FrameKind::Paragraph => html! { p class="text-gray-300" { (render_buffer(&frame.buffer)) } },
        FrameKind::Heading(level) => match level {
            HeadingLevel::H1 => html! {
                h1 class="text-4xl md:text-5xl font-semibold tracking-tight text-white mt-10 mb-6" {
                    (render_buffer(&frame.buffer))
                }
            },
            HeadingLevel::H2 => html! {
                h2 class="text-2xl md:text-3xl font-semibold tracking-tight text-white mt-10 mb-4" {
                    (render_buffer(&frame.buffer))
                }
            },
            _ => html! {
                h3 class="text-xl md:text-2xl font-semibold text-white mt-8 mb-3" {
                    (render_buffer(&frame.buffer))
                }
            },
        },
        FrameKind::BlockQuote => html! { blockquote class="text-gray-300" { (render_buffer(&frame.buffer)) } },
        FrameKind::CodeBlock(_info) => html! {
            pre { code { (render_buffer(&frame.buffer)) } }
        },
        FrameKind::List(start) => match start {
            Some(start) => html! { ol start=(start) { (render_buffer(&frame.buffer)) } },
            None => html! { ul { (render_buffer(&frame.buffer)) } },
        },
        FrameKind::Item => html! { li class="text-gray-300" { (render_buffer(&frame.buffer)) } },
        FrameKind::Emphasis => html! { em { (render_buffer(&frame.buffer)) } },
        FrameKind::Strong => html! { strong { (render_buffer(&frame.buffer)) } },
        FrameKind::Strikethrough => html! { del { (render_buffer(&frame.buffer)) } },
        FrameKind::Link { dest_url, title } => {
            if title.is_empty() {
                html! { a href=(dest_url) { (render_buffer(&frame.buffer)) } }
            } else {
                html! { a href=(dest_url) title=(title) { (render_buffer(&frame.buffer)) } }
            }
        }
        FrameKind::Image {
            dest_url,
            title,
            alt,
        } => {
            if title.is_empty() {
                html! { img src=(dest_url) alt=(alt); }
            } else {
                html! { img src=(dest_url) alt=(alt) title=(title); }
            }
        }
        FrameKind::Table => html! { table { (render_buffer(&frame.buffer)) } },
        FrameKind::TableHead => html! { thead { (render_buffer(&frame.buffer)) } },
        FrameKind::TableRow => html! { tr { (render_buffer(&frame.buffer)) } },
        FrameKind::TableCell => html! { td { (render_buffer(&frame.buffer)) } },
    }
}

pub async fn get_post(Path(id): Path<String>) -> (StatusCode, Markup) {
    let id = id.to_lowercase();
    let post = if let Some(post) = get_post_by_id(&id) {
        post
    } else {
        return (StatusCode::NOT_FOUND, pages::not_found().await.1);
    };
    (
        StatusCode::OK,
        layout(
            post.meta.title,
            html! {
                (render_post(post))
            },
        ),
    )
}

pub fn get_posts() -> &'static Vec<&'static Post> {
    Posts::published_posts()
}
