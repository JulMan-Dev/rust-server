use urlencoding::decode;

#[derive(Debug, Clone)]
pub struct RequestCookie(pub String, pub String);

#[derive(Debug, Clone)]
pub enum ParseError {
    InvalidCookiePair,
    InvalidCookieName,
    InvalidCookieValue,
}

impl RequestCookie {
    pub fn parse(raw: String) -> Result<Vec<RequestCookie>, ParseError> {
        let mut split = raw.split("; ");
        let mut out: Vec<RequestCookie> = vec![];

        while let Some(cookie_raw) = split.next() {
            let pair: Vec<&str> = cookie_raw.split("=").collect();

            if pair.len() != 2 {
                return Err(ParseError::InvalidCookiePair);
            }

            let name = pair[0].to_string();
            let value = pair[1].to_string();

            if name.is_empty() {
                return Err(ParseError::InvalidCookieName);
            }

            let value = match decode(&value) {
                Ok(value) => value.to_string(),
                Err(_) => return Err(ParseError::InvalidCookieValue),
            };

            out.push(RequestCookie(name, value));
        }

        return Ok(out);
    }

    pub fn name(&self) -> &String {
        &self.0
    }

    pub fn value(&self) -> &String {
        &self.1
    }
}

#[derive(Debug, Clone)]
pub struct ResponseCookie {
    pub name: String,
    pub value: String,
    pub max_age: Option<u64>,
    pub expires: Option<String>,
    pub path: Option<String>,
    pub domain: Option<String>,
    pub secure: bool,
    pub http_only: bool,
}

impl ToString for ResponseCookie {
    fn to_string(&self) -> String {
        let mut out = String::new();

        out.push_str(&format!("{}={};", self.name, self.value));

        if let Some(ref max_age) = self.max_age {
            out.push_str(&format!("Max-Age={};", max_age));
        }
        if let Some(ref expires) = self.expires {
            out.push_str(&format!("Expires={};", expires));
        }
        if let Some(ref path) = self.path {
            out.push_str(&format!("Path={};", path));
        }
        if let Some(ref domain) = self.domain {
            out.push_str(&format!("Domain={};", domain));
        }
        if self.secure {
            out.push_str("Secure;");
        }
        if self.http_only {
            out.push_str("HttpOnly;");
        }
        out.pop();
        out
    }
}
