use maud::{DOCTYPE, Markup, html};

pub fn layout(title: &str, content: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                title { (title) }
                ;
                link rel="stylesheet" href="/style/index.css";
                link rel="icon" type="image/png" href="/img/avatar.png";
            }
            body class="min-h-screen flex flex-col" {
                main class="flex-1" {
                    div class="mb-6 flex flex-wrap items-center gap-3 text-white" {
                        h1 class="text-3xl font-semibold" { "Lucas' Hut" }
                        span class="text-white/70 font-light -translate-y-0.5" { "/" }
                        nav class="flex items-center gap-4 text-base -translate-y-0.5" {
                            a class="border-b-0 no-underline" href="/" { "Home" }
                            a class="border-b-0 no-underline" href="/posts" { "Posts" }
                            a class="border-b-0 no-underline" href="/about" { "About" }
                        }
                    }
                    (content)
                }
                footer
                    class="site-footer mt-10 py-6 text-sm text-gray-400 flex flex-wrap justify-center text-center"
                {
                    a   href="https://bsky.app/profile/ae2.rs"
                        target="_blank"
                        rel="noopener noreferrer"
                    { "ae2.rs on Bluesky" }
                    span class="footer-separator" { "|" }
                    a href="mailto:lucas@decastro.one" { "lucas@decastro.one" }
                    span class="footer-separator" { "|" }
                    a href="/rss.xml" target="_blank" rel="noopener noreferrer" { "RSS" }
                }
            }
        }
    }
}
