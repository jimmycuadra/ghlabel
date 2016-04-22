use std::io::Error as IoError;
use std::io::Read;

use hyper::Client as HyperClient;
use hyper::Error as HyperError;
use hyper::header::{Bearer, Authorization, Headers, UserAgent};
use hyper::method::Method;
use hyper::status::StatusCode;
use rustc_serialize::json;
use rustc_serialize::json::DecoderError;


use label::Label;

#[derive(Debug)]
pub enum Error {
    Hyper(HyperError),
    Io(IoError),
    Json(DecoderError),
    NotOk(String),
}

impl From<HyperError> for Error {
    fn from(error: HyperError) -> Error {
        Error::Hyper(error)
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::Io(error)
    }
}

pub struct Client<'a> {
    client: HyperClient,
    repo: &'a str,
    token: &'a str,
    user: &'a str,
    endpoint: &'a str,
}

impl<'a> Client<'a> {
    pub fn new(repo: &'a str, token: &'a str, user: &'a str, endpoint: &'a str) -> Client<'a> {
        Client {
            client: HyperClient::new(),
            repo: repo,
            token: token,
            user: user,
            endpoint: endpoint,
        }
    }

    pub fn create<'b>(&self, label: &'b Label) -> Result<(), Error> {
        let data = self.to_json_string(label);

        let mut response = try!(
            self.client.post(
                &format!("{}/repos/{}/{}/labels", self.endpoint, self.user, self.repo)
            ).headers(self.headers()).body(&data).send()
        );

        let mut body = String::new();
        try!(response.read_to_string(&mut body));

        match response.status {
            StatusCode::Created => Ok(()),
            _ => return Err(Error::NotOk(body)),
        }
    }

    pub fn delete<'b>(&self, label: &'b Label) -> Result<(), Error> {
        let url = label.url.to_string();

        let mut response = try!(
            self.client.delete(&url).headers(self.headers()).send()
        );

        let mut body = String::new();
        try!(response.read_to_string(&mut body));

        match response.status {
            StatusCode::NoContent => Ok(()),
            _ => return Err(Error::NotOk(body)),
        }
    }

    pub fn list<'b>(&self) -> Result<Vec<Label>, Error> {
        let mut response =  try!(
            self.client.get(
                &format!("{}/repos/{}/{}/labels", self.endpoint, self.user, self.repo)
            ).headers(self.headers()).send()
        );

        let mut body = String::new();
        try!(response.read_to_string(&mut body));

        match response.status {
            StatusCode::Ok => {},
            _ => return Err(Error::NotOk(body)),
        }

        match json::decode(&body) {
            Ok(labels) => Ok(labels),
            Err(error) => Err(Error::Json(error)),
        }
    }

    pub fn update<'b>(&self, label: &'b Label) -> Result<(), Error> {
        let url = label.url.to_string();
        let data = self.to_json_string(label);

        let mut response = try!(
            self.client.request(Method::Patch, &url).headers(self.headers()).body(&data).send()
        );

        let mut body = String::new();
        try!(response.read_to_string(&mut body));

        match response.status {
            StatusCode::Ok => Ok(()),
            _ => return Err(Error::NotOk(body)),
        }
    }

    fn headers(&self) -> Headers {
        let mut headers = Headers::new();

        headers.set(Authorization(Bearer { token: self.token.to_string() }));
        headers.set(UserAgent(self.user.to_string()));

        headers
    }

    fn to_json_string<'b>(&self, label: &'b Label) -> String {
        format!("{{\"name\": \"{}\",\"color\":\"{}\"}}", label.name, label.color)
    }
}
