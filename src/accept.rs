use crate::response::BodyEncoding;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct AcceptEncodings(Vec<AcceptEncoding>);

impl AcceptEncodings {
    pub fn new(encodings: Vec<AcceptEncoding>) -> Self {
        AcceptEncodings(encodings)
    }

    pub fn accept(&self, encoding: &BodyEncoding) -> bool {
        for accept in &self.0 {
            if accept.accept(encoding) {
                return true;
            }
        }
        false
    }
}

impl FromStr for AcceptEncodings {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut encodings = Vec::new();
        let mut split = s.split(",");
        while let Some(encoding) = split.next() {
            let encoding = encoding.trim();
            if encoding.is_empty() {
                continue;
            }
            let encoding: AcceptEncoding = encoding.parse()?;
            encodings.push(encoding);
        }
        Ok(AcceptEncodings::new(encodings))
    }
}

impl Default for AcceptEncodings {
    fn default() -> Self {
        AcceptEncodings(Vec::new())
    }
}

impl ToString for AcceptEncodings {
    fn to_string(&self) -> String {
        let mut result = String::new();

        for encoding in self.0.iter() {
            result.push_str(&format!("{}, ", encoding.to_string()));
        }

        result.pop();
        result.pop();

        result
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Encoding {
    Gzip,
    Deflate,
    Br,
    All,
}

impl FromStr for Encoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "gzip" => Ok(Encoding::Gzip),
            "deflate" => Ok(Encoding::Deflate),
            "br" => Ok(Encoding::Br),
            "*" => Ok(Encoding::All),
            _ => Err(()),
        }
    }
}

impl From<BodyEncoding> for Encoding {
    fn from(encoding: BodyEncoding) -> Self {
        match encoding {
            BodyEncoding::Gzip => Encoding::Gzip,
            BodyEncoding::Deflate => Encoding::Deflate,
            BodyEncoding::Brotli => Encoding::Br,
        }
    }
}

impl ToString for Encoding {
    fn to_string(&self) -> String {
        match self {
            Encoding::Gzip => "gzip",
            Encoding::Deflate => "deflate",
            Encoding::Br => "br",
            Encoding::All => "*",
        }
        .to_string()
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AcceptEncoding {
    encoding: Encoding,
    q: Option<i8>,
}

impl AcceptEncoding {
    pub fn new(encoding: Encoding, q: Option<i8>) -> Self {
        AcceptEncoding { encoding, q }
    }

    pub fn encoding(&self) -> Encoding {
        self.encoding
    }

    pub fn quality(&self) -> Option<i8> {
        self.q
    }

    pub fn accept(&self, encoding: &BodyEncoding) -> bool {
        match (self.encoding, encoding) {
            (Encoding::All, _) => true,
            (Encoding::Gzip, BodyEncoding::Gzip) => true,
            (Encoding::Deflate, BodyEncoding::Deflate) => true,
            (Encoding::Br, BodyEncoding::Brotli) => true,
            _ => false,
        }
    }
}

impl FromStr for AcceptEncoding {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(";");
        let encoding = split.next().ok_or(())?.trim();
        let encoding: Encoding = encoding.parse()?;
        let q: Option<i8> = split
            .next()
            .and_then(|s| s.split("=").nth(1).and_then(|s| s.parse().ok()));
        Ok(AcceptEncoding::new(encoding, q))
    }
}

impl ToString for AcceptEncoding {
    fn to_string(&self) -> String {
        let mut result = String::new();

        result.push_str(&self.encoding.to_string());
        if let Some(q) = self.q {
            result.push_str(&format!(";q={}", q));
        }
        result
    }
}
