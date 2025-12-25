use proc_macro::TokenStream;
use quote::quote;
use std::fs;
use std::path::{Path, PathBuf};
use syn::{DeriveInput, LitStr, parse_macro_input};

fn strip_quotes(value: &str) -> &str {
    let bytes = value.as_bytes();
    if bytes.len() >= 2 {
        let first = bytes[0];
        let last = bytes[bytes.len() - 1];
        if (first == b'\'' && last == b'\'') || (first == b'"' && last == b'"') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

fn parse_date(value: &str) -> Option<(u16, u8, u8)> {
    let bytes = value.as_bytes();
    if bytes.len() != 10 {
        return None;
    }
    for (idx, ch) in bytes.iter().copied().enumerate() {
        match idx {
            4 | 7 => {
                if ch != b'-' {
                    return None;
                }
            }
            _ => {
                if !ch.is_ascii_digit() {
                    return None;
                }
            }
        }
    }
    let year: u16 = value[0..4].parse().ok()?;
    let month: u8 = value[5..7].parse().ok()?;
    let day: u8 = value[8..10].parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

struct PostData {
    id: String,
    title: String,
    year: u16,
    month: u8,
    day: u8,
    draft: bool,
    markdown: String,
    html: String,
}

fn parse_post(id: String, path: &Path) -> PostData {
    let content = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("failed to read post file {}: {}", path.display(), err));

    let mut lines = content.lines();
    let first = lines.next().unwrap_or_default();
    if first.trim() != "---" {
        panic!(
            "post {} must start with front matter delimited by ---",
            path.display()
        );
    }

    let mut front_lines = Vec::new();
    let mut body_lines = Vec::new();
    let mut in_front = true;
    for line in lines {
        if in_front {
            if line.trim() == "---" {
                in_front = false;
                continue;
            }
            front_lines.push(line);
        } else {
            body_lines.push(line);
        }
    }

    if in_front {
        panic!("post {} front matter must end with ---", path.display());
    }

    let mut title: Option<String> = None;
    let mut published: Option<String> = None;
    let mut draft: Option<bool> = None;

    for line in front_lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once(':')
            .unwrap_or_else(|| panic!("invalid front matter line in {}: {}", path.display(), line));
        let key = key.trim();
        let raw_value = strip_quotes(value.trim());
        match key {
            "title" => title = Some(raw_value.to_string()),
            "published" => published = Some(raw_value.to_string()),
            "draft" => {
                let parsed = match raw_value {
                    "true" => true,
                    "false" => false,
                    _ => panic!("draft must be true or false in {}", path.display()),
                };
                draft = Some(parsed);
            }
            _ => {}
        }
    }

    let title = title.unwrap_or_else(|| {
        panic!(
            "post {} is missing required front matter: title",
            path.display()
        )
    });
    let published = published.unwrap_or_else(|| {
        panic!(
            "post {} is missing required front matter: published",
            path.display()
        )
    });
    let (year, month, day) = parse_date(&published).unwrap_or_else(|| {
        panic!(
            "post {} has invalid published date (expected YYYY-MM-DD)",
            path.display()
        )
    });
    let draft = draft.unwrap_or_else(|| {
        panic!(
            "post {} is missing required front matter: draft",
            path.display()
        )
    });

    let markdown = body_lines.join("\n");
    let html = markdown::to_html(&markdown);

    PostData {
        id,
        title,
        year,
        month,
        day,
        draft,
        markdown,
        html,
    }
}

#[proc_macro_derive(Post)]
pub fn derive_post(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let content_dir = PathBuf::from(manifest_dir).join("content").join("post");

    let entries = fs::read_dir(&content_dir).unwrap_or_else(|err| {
        panic!(
            "failed to read post directory {}: {}",
            content_dir.display(),
            err
        )
    });

    let mut post_dirs: Vec<PathBuf> = entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect();
    post_dirs.sort();

    let mut posts = Vec::new();
    for dir in post_dirs {
        let id = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let index_path = dir.join("index.md");
        if !index_path.exists() {
            panic!("post directory {} is missing index.md", dir.display());
        }
        posts.push(parse_post(id, &index_path));
    }

    let insertions = posts.iter().map(|post| {
        let id_lit = LitStr::new(&post.id, name.span());
        let title_lit = LitStr::new(&post.title, name.span());
        let markdown_lit = LitStr::new(&post.markdown, name.span());
        let html_lit = LitStr::new(&post.html, name.span());
        let year = post.year;
        let month = post.month;
        let day = post.day;
        let draft = post.draft;

        quote! {
            map.insert(
                #id_lit,
                crate::content::meta::Post {
                    id: #id_lit,
                    meta: crate::content::meta::PostMeta {
                        title: #title_lit,
                        published: crate::content::meta::Date {
                            year: #year,
                            month: #month,
                            day: #day,
                        },
                        draft: #draft,
                    },
                    markdown: #markdown_lit,
                    html: #html_lit,
                },
            );
        }
    });

    let expanded = quote! {
        impl #name {
            fn map() -> &'static std::collections::HashMap<&'static str, crate::content::meta::Post> {
                static POSTS: std::sync::OnceLock<
                    std::collections::HashMap<&'static str, crate::content::meta::Post>,
                > = std::sync::OnceLock::new();
                POSTS.get_or_init(|| {
                    let mut map = std::collections::HashMap::new();
                    #(#insertions)*
                    map
                })
            }

            pub fn get(title: &str) -> Option<&'static crate::content::meta::Post> {
                Self::map().get(title)
            }

            pub fn iter(
            ) -> impl Iterator<Item = &'static crate::content::meta::Post> {
                Self::map().values()
            }
        }
    };

    TokenStream::from(expanded)
}
