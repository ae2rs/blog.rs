use maud::Markup;
use pulldown_cmark::HeadingLevel;

#[derive(Clone, Copy)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Ord for Date {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.year.cmp(&other.year).then_with(|| {
            self.month
                .cmp(&other.month)
                .then_with(|| self.day.cmp(&other.day))
        })
    }
}

impl PartialOrd for Date {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Date {
    fn eq(&self, other: &Self) -> bool {
        self.year == other.year && self.month == other.month && self.day == other.day
    }
}

impl Eq for Date {}

#[derive(Clone, Copy)]
pub struct PostMeta {
    pub title: &'static str,
    pub published: Date,
    pub draft: bool,
}

#[derive(Clone, Copy)]
pub struct Post {
    pub id: &'static str,
    pub meta: PostMeta,
    pub markdown: &'static str,
    pub events: fn() -> pulldown_cmark::TextMergeStream<'static, pulldown_cmark::Parser<'static>>,
}
#[derive(Debug)]
pub struct Frame {
    pub kind: FrameKind,
    pub buffer: Vec<RenderNode>,
    pub text: String,
}

#[derive(Debug)]
pub enum FrameKind {
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
pub enum RenderNode {
    Markup(Markup),
    CodeBlock { info: Option<String>, text: String },
}
