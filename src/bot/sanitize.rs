use regex::Regex;
use url::Url;

// https://stackoverflow.com/questions/6038061/regular-expression-to-find-urls-within-a-string
const RE_URL: &str =
    r"(http|ftp|https):\/\/([\w_-]+(?:(?:\.[\w_-]+)+))([\w.,@?^=%&:\/~+#-]*[\w@?^=%&\/~+#-])";

pub fn extract_url(text: &str) -> Option<&str> {
    let re = Regex::new(RE_URL).unwrap();
    match re.find(text) {
        Some(m) => Some(m.as_str()),
        None => None
    }
}

pub fn parse_url(url: &str) -> Option<Url> {
    Url::parse(url).ok()
}

#[cfg(test)]
mod tests {
    use crate::bot::sanitize::{extract_url, parse_url};

    #[test]
    fn test_extract_url() {
        assert_eq!(extract_url("test http://www.test.com/id/1"), Some("http://www.test.com/id/1"));
        assert_eq!(extract_url("https://www.test.com 3"), Some("https://www.test.com"));
        assert_eq!(extract_url("there is no any url"), None);
    }

    #[test]
    fn test_parse_url() {
        let url = parse_url("https://www.youtube.com/watch?v=00000000000").unwrap();
        assert_eq!(url.host_str().unwrap(), "www.youtube.com");

        let url = parse_url("https://youtu.be/00000000000").unwrap();
        assert_eq!(url.host_str().unwrap(), "youtu.be");
    }
}