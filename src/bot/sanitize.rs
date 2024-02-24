use regex::Regex;

// https://stackoverflow.com/questions/6038061/regular-expression-to-find-urls-within-a-string
const RE_URL: &str =
    r"(http|ftp|https):\/\/([\w_-]+(?:(?:\.[\w_-]+)+))([\w.,@?^=%&:\/~+#-]*[\w@?^=%&\/~+#-])";

pub fn extract_urls(text: &str) -> Vec<&str> {
    let re = Regex::new(RE_URL).unwrap();
    re.find_iter(text)
        .map(|m| m.as_str())
        .collect::<Vec<&str>>()
}
