use chrono::{Duration, DateTime, Utc};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::io::Read;

const STALE_AFTER_DAYS:i64 = 62;

pub fn may_index(url: &str) -> bool {
    let html = fetch(url);
    meta_robots_is_allowed(&html)
}

pub fn is_stale(created_at: &DateTime<Utc>) -> bool {
    let ago = Utc::now() - Duration::days(STALE_AFTER_DAYS);
    created_at < &ago
}

fn fetch(url: &str) -> String {
    let mut body = String::new();

    if let Ok(mut res) = Client::new().get(url).send() {
        res.read_to_string(&mut body).ok();
    }

    body
}

fn meta_robots_is_allowed(html: &str) -> bool {
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
    use chrono::{offset, Duration};

    #[test]
    fn determine_meta_robots_has_no_meta_tag() {
        let html = "<head><title>Title</title></head>".to_string();
        assert!(meta_robots_is_allowed(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_no_noindex() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"index\"/></head>"
            .to_string();
        assert!(meta_robots_is_allowed(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_no_content() {
        let html = "<head><title>Title</title><meta name=\"robots\"/></head>".to_string();
        assert!(meta_robots_is_allowed(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_none() {
        let html =
            "<head><title>Title</title><meta name=\"robots\" content=\"none\"/></head>".to_string();
        assert!(!meta_robots_is_allowed(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_noindex() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"noindex\"/></head>"
            .to_string();
        assert!(!meta_robots_is_allowed(&html));
    }

    #[test]
    fn determine_meta_robots_has_meta_tag_several() {
        let html = "<head><title>Title</title><meta name=\"robots\" content=\"nosnippet noindex nofollow\"/></head>".to_string();
        assert!(!meta_robots_is_allowed(&html));
    }

    #[test]
    fn is_stale_is_false_with_recent_dt() {
        let dt = offset::Utc::now();
        assert!(!is_stale(&dt));
    }

    #[test]
    fn is_stale_is_true_with_old_dt() {
        let dt = offset::Utc::now() - Duration::days(STALE_AFTER_DAYS + 1);
        assert!(is_stale(&dt));
    }
}
