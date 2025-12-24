use crate::common::layout;
use maud::{Markup, html};

pub async fn page() -> Markup {
    let content = html! {
        p {
            "I've been coding and making software for the last ten years, and even if us SWEs tend to complain a lot, I just love my job."
        }
        p {
            "To me, coding is a craftsmanship, and following this analogy, I strive on becoming a "
            a href="https://en.wikipedia.org/wiki/Master_craftsman" { "master craftsman" }
            ". This blog is a new way for me to share my passion, a mean to "
            a href="https://www.benkuhn.net/writing/" { "make awsome new friends" }
            " with similar interests."
        }
        p {
            "This "
            a href="https://github.com/ae2rs/blog" { "open source" }
            " blog is also an excuse for me to fulfill my second passion: teaching. Personally, the way I like to learn is from "
            em { "someone" }
            " instead of "
            em { "something" }
            ", and I think it's time for me to start "
            a href="https://pcandmore.net/blog/producing-instead-of-consuming/" { "producing after consuming so much" }
            "."
        }
    };

    layout("About", content)
}
