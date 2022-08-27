#[derive(Debug, Clone)]
pub enum Mime {
    Custom(String, String, Option<(String, String)>),
    Text(String, Option<(String, String)>),
    Application(String, Option<(String, String)>),
    Audio(String, Option<(String, String)>),
    Image(String, Option<(String, String)>),
    Message(String, Option<(String, String)>),
    Model(String, Option<(String, String)>),
    Video(String, Option<(String, String)>),
}

impl Mime {
    pub fn new(type_: String, subtype: String, parameters: Option<(String, String)>) -> Mime {
        match type_.as_str() {
            "text" => Mime::Text(subtype, parameters),
            "application" => Mime::Application(subtype, parameters),
            "audio" => Mime::Audio(subtype, parameters),
            "image" => Mime::Image(subtype, parameters),
            "message" => Mime::Message(subtype, parameters),
            "model" => Mime::Model(subtype, parameters),
            "video" => Mime::Video(subtype, parameters),
            _ => Mime::Custom(type_, subtype, parameters),
        }
    }

    pub fn from_extension(extension: &String, default: Option<Mime>) -> Option<Mime> {
        let mut s = extension.clone();

        while s.starts_with(".") {
            s.remove(0);
        }

        let found = match s.as_str() {
            "txt" => Some("text/plain"),
            "html" => Some("text/html"),
            "js" => Some("application/javascript"),
            "mp3" => Some("audio/mp3"),
            "mp4" => Some("video/mp3"),
            _ => None,
        };

        if let Some(raw_mime) = found {
            match Mime::parse(&raw_mime.to_string()) {
                Ok(mime) => Some(mime),
                Err(_) => None.or(default),
            }
        } else {
            None.or(default)
        }
    }

    pub fn parse(raw: &String) -> Result<Mime, String> {
        let mut split = raw.split("/");
        let type_ = match split.next() {
            Some(type_) => type_.to_string(),
            None => return Err("Invalid MIME type".to_string()),
        };

        let (subtype, parameters) = {
            let mut sub = match split.next() {
                Some(sub) => sub.to_string(),
                None => return Err("Invalid MIME type".to_string()),
            };

            let mut parameters = (None, None);

            if sub.contains(";") {
                let sub_clone = sub.clone();
                let mut split = sub_clone.split(";");
                sub = split.next().unwrap().to_string();

                let param = match split.next() {
                    Some(param) => param.to_string(),
                    None => return Err("Invalid MIME type".to_string()),
                };
                let mut param_split = param.split("=");
                let key = param_split.next().unwrap().to_string();
                let value = param_split.next().unwrap().to_string();

                parameters = (Some(key), Some(value));
            }

            (sub, parameters)
        };

        let parameters = match parameters {
            (Some(key), Some(value)) => Some((key, value)),
            _ => None,
        };

        Ok(Mime::new(type_, subtype, parameters))
    }

    pub fn custom(type_: &str, subtype: &str) -> Mime {
        Mime::Custom(String::from(type_), String::from(subtype), None)
    }

    pub fn text(subtype: &str) -> Mime {
        Mime::Text(String::from(subtype), None)
    }

    pub fn application(subtype: &str) -> Mime {
        Mime::Application(String::from(subtype), None)
    }

    pub fn audio(subtype: &str) -> Mime {
        Mime::Audio(String::from(subtype), None)
    }

    pub fn image(subtype: &str) -> Mime {
        Mime::Image(String::from(subtype), None)
    }

    pub fn message(subtype: &str) -> Mime {
        Mime::Message(String::from(subtype), None)
    }

    pub fn model(subtype: &str) -> Mime {
        Mime::Model(String::from(subtype), None)
    }

    pub fn video(subtype: &str) -> Mime {
        Mime::Video(String::from(subtype), None)
    }
}

impl ToString for Mime {
    fn to_string(&self) -> String {
        let (type_, subtype, parameters) = {
            let val = String::from(match self {
                Mime::Custom(type_, _, _) => type_.as_str(),
                Mime::Text(_, _) => "text",
                Mime::Application(_, _) => "application",
                Mime::Audio(_, _) => "audio",
                Mime::Image(_, _) => "image",
                Mime::Message(_, _) => "message",
                Mime::Model(_, _) => "model",
                Mime::Video(_, _) => "video",
            });

            match self {
                Mime::Custom(type_, subtype, parameters) => (type_.clone(), subtype, parameters),
                Mime::Text(subtype, parameters) => (val, subtype, parameters),
                Mime::Application(subtype, parameters) => (val, subtype, parameters),
                Mime::Audio(subtype, parameters) => (val, subtype, parameters),
                Mime::Image(subtype, parameters) => (val, subtype, parameters),
                Mime::Message(subtype, parameters) => (val, subtype, parameters),
                Mime::Model(subtype, parameters) => (val, subtype, parameters),
                Mime::Video(subtype, parameters) => (val, subtype, parameters),
            }
        };

        let mut out = String::new();
        out.push_str(&type_);
        out.push_str("/");
        out.push_str(&subtype);

        if let Some((key, value)) = parameters {
            out.push_str(";");
            out.push_str(&key);
            out.push_str("=");
            out.push_str(&value);
        }

        out
    }
}
