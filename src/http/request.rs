// Super means: go 1 level up to parent
use super::method::{Method, MethodError};
use super::{QueryString, QueryStringValue};
use std::convert::TryFrom;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::str;
use std::str::Utf8Error;

// We have to explicitly specify a lifetime for every reference that we store inside of a struct
#[derive(Debug)]
pub struct Request<'buf> {
    path: &'buf str,
    // Express the absence of a value via Option
    // -    TakesEither `None` or `Some(String)`
    query_string: Option<QueryString<'buf>>,
    method: Method,
}

impl<'buf> Request<'buf> {
    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    // We can change type we store for query_string and wrap it in option in the getter
    pub fn query_string(&self) -> Option<&QueryString> {
        self.query_string.as_ref()
    }
}

// using 'buf lifetime to guarantee compiler memory safety &
// doesn't allow us to choose how long a value lives,
// but to communicate to the compiler that some references are related to the same memory and are expected to share the same lifetime
impl<'buf> TryFrom<&'buf [u8]> for Request<'buf> {
    type Error = ParseError;

    // e.g., GET /search?name=abc&sort=1 HTTP/1.1\r\n...HEADERS...
    fn try_from(buf: &'buf [u8]) -> Result<Request<'buf>, Self::Error> {
        // Makes sure bytes in buf are UTF
        let request = str::from_utf8(buf)?;

        // transforms option into result by looking at option:
        // -    if option is Some, convert to Ok variant of result
        // -    else if none, return err wrapping param

        // First call == `GET`
        let (method, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;

        // Second call == `/search?name=abc&sort=1`
        let (mut path, request) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;

        // Third call == `HTTP/1.1`
        let (protocol, _) = get_next_word(request).ok_or(ParseError::InvalidRequest)?;

        if protocol != "HTTP/1.1" {
            return Err(ParseError::InvalidProtocol);
        }

        // convert type into another type (e.g, String to Enum)
        let method: Method = method.parse()?;

        // returns the byte idnex of the  first character of the first matching string slice
        let mut query_string = None;
        if let Some(i) = path.find('?') {
            query_string = Some(QueryString::from(&path[i + 1..]));
            // path = everything before '?'
            path = &path[..i];
        }

        Ok(Self {
            path,
            query_string,
            method,
        })
    }
}

// e.g, for:  GET /search?name=abc&sort=1 HTTP/1.1
// 1. GET
// 2. pass in: /search?name=abc&sort=1 HTTP/1.1
// 2. /search?name=abc&sort=1
// 4. HTTP/1.1
fn get_next_word(request: &str) -> Option<(&str, &str)> {
    // loop through each character element in `request`
    // enumerate() gives `index, value`
    for (i, c) in request.chars().enumerate() {
        if c == ' ' || c == '\r' {
            // get all characters until index `i` (all characters before space)
            return Some((&request[..i], &request[i + 1..]));
        }
    }

    None
}

// Different errors we may encounter
pub enum ParseError {
    InvalidRequest,  // General error
    InvalidEncoding, // Not UTF encoded
    InvalidProtocol, // Requests that have invalid http version
    InvalidMethod,   // Not one of the methods in enum
}

impl ParseError {
    fn message(&self) -> &str {
        match self {
            Self::InvalidRequest => "Invalid Request",
            Self::InvalidEncoding => "Invalid Encoding",
            Self::InvalidProtocol => "Invalid Protocol",
            Self::InvalidMethod => "Invalid Method",
        }
    }
}

impl From<MethodError> for ParseError {
    fn from(_: MethodError) -> Self {
        Self::InvalidMethod
    }
}

impl From<Utf8Error> for ParseError {
    fn from(_: Utf8Error) -> Self {
        Self::InvalidEncoding
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Debug for ParseError {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "{}", self.message())
    }
}

impl Error for ParseError {}
