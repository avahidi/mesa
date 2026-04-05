
#[derive(Debug)]
pub struct Capture {
    prefix: Vec<String>,
    suffix: String,
}

// parse() expected data in format /prefix1/.../prefix n/suffix/ where / can be replaced with any marker
pub fn parse(s: &str) -> Result<Capture, String> {
    if s.len() >= 2 {
        let marker = s.chars().next().unwrap();
        let marker_str = marker.to_string();

        if s.ends_with(&marker_str) {
            let inner = &s[marker.len_utf8()..s.len() - marker.len_utf8()];
            let result: Vec<String> = inner.split(marker).map(String::from).collect();
            if result.len() == 1 {
                return Ok( Capture{prefix: result, suffix: String::new() })
            }
            if let Some((last, rest)) = result.split_last() {
                return Ok(Capture { prefix: rest.to_vec(), suffix: last.clone() });
            }
        }
    }
    Err("Capture format is ?prefix?...?suffix? where '?' is marker".to_string() )
}

impl Capture {
    pub fn extract(&self, text: &str ) -> Option<String> {
     let mut position = 0;
        for prefix in &self.prefix {
            let p = text[position..].find(prefix)?;
            position = position + p + prefix.len();
        }

        let suffix_pos = if self.suffix.is_empty() {
            text[position..].find("\n").unwrap_or(text.len() - position)
        } else {
            text[position..].find(&self.suffix)?
        };
        Some(text[position..position + suffix_pos].trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid() {
        assert!(parse("start/end/").is_err());
        assert!(parse("/a").is_err());
    }

    #[test]
    fn test_extract_short() {
        let capture = parse("/is/").unwrap();
        let text = "My uncle is old";
        let result = capture.extract(&text);
        assert_eq!(result, Some("old".to_string()));
    }

    #[test]
    fn test_extract_simple() {
        let capture = parse("/is/years/").unwrap();
        let text = "My uncle is 100 years old";
        let result = capture.extract(&text);
        assert_eq!(result, Some("100".to_string()));
    }

    #[test]
    fn test_extract_series() {
        let capture = parse("/is/is/years/").unwrap();
        let text = "My uncle is 100 years old, but my other uncle is 110 years old.";
        let result = capture.extract(&text);
        assert_eq!(result, Some("110".to_string()));
    }
}

