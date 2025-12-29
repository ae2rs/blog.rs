use maud::{DOCTYPE, Markup, html};

pub fn layout(title: &str, content: Markup) -> Markup {
    layout_with_head(title, content, None)
}

pub fn layout_with_head(title: &str, content: Markup, head_extras: Option<Markup>) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                ;
                link rel="stylesheet" href="/style/index.css";
                link rel="icon" type="image/png" href="/img/avatar.png";
                @if let Some(extras) = head_extras { (extras) }
            }
            body class="min-h-screen flex flex-col" {
                main class="flex-1 w-full" {
                    div class="mb-6 flex w-full flex-wrap items-center gap-3 text-white" {
                        h1 class="text-3xl font-semibold" { "Lucas' Hut" }
                        span
                            class="text-white/70 font-light -translate-y-0.5 text-lg sm:text-base"
                        { "/" }
                        nav class="flex items-center gap-4 text-lg sm:text-base -translate-y-0.5" {
                            a class="border-b-0 no-underline" href="/" { "Home" }
                            a class="border-b-0 no-underline" href="/posts" { "Posts" }
                            a class="border-b-0 no-underline" href="/about" { "About" }
                        }
                    }
                    (content)
                }
                footer
                    class="site-footer mt-10 py-6 text-base sm:text-sm text-gray-400 flex flex-wrap justify-center text-center"
                {
                    a   href="https://bsky.app/profile/ae2.rs"
                        target="_blank"
                        rel="noopener noreferrer"
                    {
                        "ae2.rs "
                        span class="footer-extra" { "on Bluesky" }
                    }
                    span class="footer-separator" { "|" }
                    a href="mailto:lucas@decastro.one" { "lucas@decastro.one" }
                }
            }
        }
    }
}
