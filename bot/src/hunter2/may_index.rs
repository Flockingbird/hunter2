use reqwest::Client;
use scraper::{Html, Selector};
use std::io::Read;

pub fn may_index(url: &str) -> bool {
    let html = fetch(url);
    determine_meta_robots(&html)
}

fn fetch(url: &str) -> String {
    let mut body = String::new();

    if let Ok(mut res) = Client::new().get(url).send() {
        res.read_to_string(&mut body).ok();
    }

    body
}

fn determine_meta_robots(html: &str) -> bool {
    let fragment = Html::parse_fragment(html);
    let selector = Selector::parse(r#"meta[name="robots"]"#).unwrap();

    if let Some(robots_meta) = fragment.select(&selector).next() {
        if let Some(content) = robots_meta.value().attr("content") {
            !(content.contains("noindex") || content.contains("none"))
        } else {
            true
        }
    } else {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn determine_meta_robots_has_no_meta_tag() {
        let html = "<head><title>Title</title></head>".to_string();
        assert!(determine_meta_robots(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_no_noindex() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"index\"/></head>"
            .to_string();
        assert!(determine_meta_robots(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_no_content() {
        let html = "<head><title>Title</title><meta name=\"robots\"/></head>".to_string();
        assert!(determine_meta_robots(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_none() {
        let html =
            "<head><title>Title</title><meta name=\"robots\" content=\"none\"/></head>".to_string();
        assert!(!determine_meta_robots(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_noindex() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"noindex\"/></head>"
            .to_string();
        assert!(!determine_meta_robots(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_several() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"nosnippet noindex nofollow\"/></head>".to_string();
        assert!(!determine_meta_robots(&html));
    }
}
