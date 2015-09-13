use url::{ParseError, Url};

#[derive(Debug, RustcDecodable)]
pub struct Label {
    pub color: String,
    pub name: String,
    pub url: Url,
}

impl PartialEq<Label> for Label {
    fn eq(&self, other: &Label) -> bool {
        self.name == other.name
    }
}

#[derive(Debug)]
pub enum Error {
    MissingColor,
    MissingName,
    UrlParseError(ParseError),
    YamlItemNotHash,
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Error {
        Error::UrlParseError(error)
    }
}

pub type Result = ::std::result::Result<Label, Error>;

impl Label {
    pub fn new<'a>(
        name: &'a str,
        color: &'a str,
        user: &'a str,
        repo: &'a str
    ) -> Result {
        let url = try!(
            Url::parse(&format!("https://api.github.com/repos/{}/{}/labels/{}", user, repo, name))
        );

        Ok(Label {
            color: color.to_string(),
            name: name.to_string(),
            url: url,
        })
    }
}
