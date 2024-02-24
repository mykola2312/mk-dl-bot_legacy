use regex::Regex;

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

#[cfg(test)]
mod tests {
    use crate::bot::sanitize::extract_url;

    #[test]
    fn test_extract_url() {
        // https://www.youtube.com/watch?v=00000000000

        assert_eq!(extract_url("test http://www.test.com/id/1"), Some("http://www.test.com/id/1"));
        assert_eq!(extract_url("https://www.test.com 3"), Some("https://www.test.com"));
        assert_eq!(extract_url("there is no any url"), None);
    }
}