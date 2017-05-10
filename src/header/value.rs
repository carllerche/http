use bytes::Bytes;

use std::{char, cmp, fmt, str};

/// Represents an HTTP header field value.
///
/// In practice, HTTP header field values are usually valid ASCII. However, the
/// HTTP spec allows for a header value to contain opaque bytes as well. In this
/// case, the header field value is not able to be represented as a string.
///
/// To handle this, the `HeaderValue` is useable as a type and can be compared
/// with strings and implements `Debug`. A `to_str` fn is provided that returns
/// an `Err` if the header value contains non visible ascii characters.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HeaderValue {
    inner: Bytes,
}

/// A possible error when converting a `HeaderValue` from a string or byte
/// slice.
#[derive(Debug)]
pub struct InvalidValueError {
    _priv: (),
}

/// A possible error when converting a `HeaderValue` to a string representation.
///
/// Header field values may contain opaque bytes, in which case it is not
/// possible to represent the value as a string.
#[derive(Debug)]
pub struct ToStrError {
    _priv: (),
}

impl HeaderValue {
    /// Convert a static string to a `HeaderValue`
    ///
    /// This function will not perform any copying, however the string is
    /// checked to ensure that no invalid characters are present. Only visible
    /// ASCII characters (32-127) are permitted.
    ///
    /// # Panics
    ///
    /// This function panics if the argument contains invalid header value
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val, "hello");
    /// ```
    pub fn from_static(src: &'static str) -> HeaderValue {
        let bytes = src.as_bytes();

        for &b in bytes {
            if !is_visible_ascii(b) {
                panic!("invalid header value");
            }
        }

        HeaderValue {
            inner: Bytes::from_static(bytes),
        }
    }

    /// Attempt to convert a string to a `HeaderValue`.
    ///
    /// If the argument contains invalid header value characters, an error is
    /// returned. Only visible ASCII characters (32-127) are permitted. Use
    /// `try_from_bytes` to create a `HeaderValue` that includes opaque octets
    /// (128-255).
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::try_from_str("hello").unwrap();
    /// assert_eq!(val, "hello");
    /// ```
    ///
    /// An invalid value
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::try_from_str("\n");
    /// assert!(val.is_err());
    /// ```
    pub fn try_from_str(src: &str) -> Result<HeaderValue, InvalidValueError> {
        let bytes = src.as_bytes();

        for &b in bytes {
            if !is_visible_ascii(b) {
                return Err(InvalidValueError {
                    _priv: (),
                });
            }
        }

        Ok(HeaderValue { inner: Bytes::from(bytes) })
    }

    /// Attempt to convert a byte slice to a `HeaderValue`.
    ///
    /// If the argument contains invalid header value bytes, an error is
    /// returned. Only byte values between 32 and 255 (inclusive) are permitted.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::try_from_slice(b"hello\xfa").unwrap();
    /// assert_eq!(val, &b"hello\xfa"[..]);
    /// ```
    ///
    /// An invalid value
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::try_from_slice(b"\n");
    /// assert!(val.is_err());
    /// ```
    pub fn try_from_slice(src: &[u8]) -> Result<HeaderValue, InvalidValueError> {
        for &b in src {
            if !is_valid(b) {
                return Err(InvalidValueError {
                    _priv: (),
                });
            }
        }

        Ok(HeaderValue { inner: Bytes::from(src) })
    }

    /// Yields a `&str` slice if the `HeaderValue` only contains visible ASCII
    /// chars.
    ///
    /// This function will perform a scan of the header value, checking all the
    /// characters.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.to_str().unwrap(), "hello");
    /// ```
    pub fn to_str(&self) -> Result<&str, ToStrError> {
        let bytes = self.as_ref();

        for &b in bytes {
            if !is_visible_ascii(b) {
                return Err(ToStrError { _priv: () });
            }
        }

        unsafe { Ok(str::from_utf8_unchecked(bytes)) }
    }

    /// Returns the length of `self`.
    ///
    /// This length is in bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.len(), 5);
    /// ```
    pub fn len(&self) -> usize {
        self.as_ref().len()
    }

    /// Returns true if the `HeaderValue` has a length of zero bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("");
    /// assert!(val.is_empty());
    ///
    /// let val = HeaderValue::from_static("hello");
    /// assert!(!val.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts a `HeaderValue` to a byte slice.
    ///
    /// # Examples
    ///
    /// ```
    /// # use http::header::HeaderValue;
    /// let val = HeaderValue::from_static("hello");
    /// assert_eq!(val.as_bytes(), b"hello");
    /// ```
    pub fn as_bytes(&self) -> &[u8] {
        self.as_ref()
    }
}

impl AsRef<[u8]> for HeaderValue {
    fn as_ref(&self) -> &[u8] {
        self.inner.as_ref()
    }
}

impl fmt::Debug for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "\""));

        for &b in self.as_ref() {
            if is_visible_ascii(b) {
                let ch = unsafe { char::from_u32_unchecked(b as u32) };
                try!(write!(f, "{}", ch));
            } else {
                try!(write!(f, "\\x{:x}", b));
            }
        }

        write!(f, "\"")
    }
}

fn is_visible_ascii(b: u8) -> bool {
    is_valid(b) && b < 127
}

fn is_valid(b: u8) -> bool {
    b >= 32
}

// ===== PartialEq / PartialOrd =====

impl PartialEq<str> for HeaderValue {
    fn eq(&self, other: &str) -> bool {
        self.inner == other.as_bytes()
    }
}

impl PartialEq<[u8]> for HeaderValue {
    fn eq(&self, other: &[u8]) -> bool {
        self.inner == other
    }
}

impl PartialOrd<str> for HeaderValue {
    fn partial_cmp(&self, other: &str) -> Option<cmp::Ordering> {
        (*self.inner).partial_cmp(other.as_bytes())
    }
}

impl PartialOrd<[u8]> for HeaderValue {
    fn partial_cmp(&self, other: &[u8]) -> Option<cmp::Ordering> {
        (*self.inner).partial_cmp(other)
    }
}

impl PartialEq<HeaderValue> for str {
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

impl PartialEq<HeaderValue> for [u8] {
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

impl PartialOrd<HeaderValue> for str {
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl PartialOrd<HeaderValue> for [u8] {
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl PartialEq<String> for HeaderValue {
    fn eq(&self, other: &String) -> bool {
        *self == &other[..]
    }
}

impl PartialOrd<String> for HeaderValue {
    fn partial_cmp(&self, other: &String) -> Option<cmp::Ordering> {
        self.inner.partial_cmp(other.as_bytes())
    }
}

impl PartialEq<HeaderValue> for String {
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

impl PartialOrd<HeaderValue> for String {
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}

impl<'a, T: ?Sized> PartialEq<&'a T> for HeaderValue
    where HeaderValue: PartialEq<T>
{
    fn eq(&self, other: &&'a T) -> bool {
        *self == **other
    }
}

impl<'a, T: ?Sized> PartialOrd<&'a T> for HeaderValue
    where HeaderValue: PartialOrd<T>
{
    fn partial_cmp(&self, other: &&'a T) -> Option<cmp::Ordering> {
        self.partial_cmp(*other)
    }
}

impl<'a> PartialEq<HeaderValue> for &'a str {
    fn eq(&self, other: &HeaderValue) -> bool {
        *other == *self
    }
}

impl<'a> PartialOrd<HeaderValue> for &'a str {
    fn partial_cmp(&self, other: &HeaderValue) -> Option<cmp::Ordering> {
        self.as_bytes().partial_cmp(other.as_bytes())
    }
}
