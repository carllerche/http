extern crate http;

use http::*;
use http::header::*;

#[test]
fn smoke() {
    let mut headers = HeaderMap::new();

    assert!(headers.get("hello").is_none());

    let name: HeaderName = "hello".parse().unwrap();

    match headers.entry(&name) {
        Entry::Vacant(e) => {
            e.insert("world");
        }
        _ => panic!(),
    }

    assert!(headers.get("hello").is_some());

    match headers.entry(&name) {
        Entry::Occupied(mut e) => {
            assert_eq!(e.first(), &"world");

            // Push another value
            e.append("zomg");

            assert_eq!(*e.first(), "world");
            assert_eq!(*e.last(), "zomg");
        }
        _ => panic!(),
    }
}

#[test]
fn drain() {
    let mut headers = HeaderMap::new();

    // Insert a single value
    headers.insert("hello", "world");

    {
        let mut iter = headers.drain();
        let (name, values) = iter.next().unwrap();
        assert_eq!(name.as_str(), "hello");

        let values: Vec<_> = values.collect();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "world");

        assert!(iter.next().is_none());
    }

    assert!(headers.is_empty());

    // Insert two sequential values
    headers.insert("hello", "world");
    headers.insert("zomg", "bar");
    headers.append("hello", "world2");

    // Drain...
    {
        let mut iter = headers.drain();
        let (name, values) = iter.next().unwrap();
        assert_eq!(name.as_str(), "hello");

        let values: Vec<_> = values.collect();
        assert_eq!(values.len(), 2);
        assert_eq!(values[0], "world");
        assert_eq!(values[1], "world2");

        let (name, values) = iter.next().unwrap();
        assert_eq!(name.as_str(), "zomg");

        let values: Vec<_> = values.collect();
        assert_eq!(values.len(), 1);
        assert_eq!(values[0], "bar");

        assert!(iter.next().is_none());
    }
}

#[test]
fn drain_entry() {
    let mut headers = HeaderMap::new();

    headers.insert("hello", "world");
    headers.insert("zomg", "foo");
    headers.append("hello", "world2");
    headers.insert("more", "words");
    headers.append("more", "insertions");

    // Using insert
    {
        let vals: Vec<_> = headers.insert("hello", "wat").unwrap().collect();
        assert_eq!(2, vals.len());
        assert_eq!(vals[0], "world");
        assert_eq!(vals[1], "world2");
    }
}

#[test]
fn eq() {
    let mut a = HeaderMap::new();
    let mut b = HeaderMap::new();

    assert_eq!(a, b);

    a.insert("hello", "world");
    assert_ne!(a, b);

    b.insert("hello", "world");
    assert_eq!(a, b);

    a.insert("foo", "bar");
    a.append("foo", "baz");
    assert_ne!(a, b);

    b.insert("foo", "bar");
    assert_ne!(a, b);

    b.append("foo", "baz");
    assert_eq!(a, b);

    a.append("a", "a");
    a.append("a", "b");
    b.append("a", "b");
    b.append("a", "a");

    assert_ne!(a, b);
}

#[test]
fn insert_all_std_headers() {
    let mut m = HeaderMap::new();

    for (i, hdr) in STD.iter().enumerate() {
        m.insert(hdr.clone(), hdr.as_str().to_string());

        for j in 0..(i+1) {
            assert_eq!(m[&STD[j]], STD[j].as_str());
        }

        if i != 0 {
            for j in (i+1)..STD.len() {
                assert!(m.get(&STD[j]).is_none(), "contained {}; j={}", STD[j].as_str(), j);
            }
        }
    }
}

#[test]
fn insert_79_custom_std_headers() {
    let mut h = HeaderMap::new();
    let hdrs = custom_std(79);

    for (i, hdr) in hdrs.iter().enumerate() {
        h.insert(hdr.clone(), hdr.as_str().to_string());

        for j in 0..(i+1) {
            assert_eq!(h[&hdrs[j]], hdrs[j].as_str());
        }

        for j in (i+1)..hdrs.len() {
            assert!(h.get(&hdrs[j]).is_none());
        }
    }
}

fn custom_std(n: usize) -> Vec<HeaderName> {
    (0..n).map(|i| {
        let s = format!("{}-{}", STD[i % STD.len()].as_str(), i);
        s.parse().unwrap()
    }).collect()
}

const STD: [HeaderName; 79] = [
    ACCEPT,
    ACCEPT_CHARSET,
    ACCEPT_ENCODING,
    ACCEPT_LANGUAGE,
    ACCEPT_PATCH,
    ACCEPT_RANGES,
    ACCESS_CONTROL_ALLOW_CREDENTIALS,
    ACCESS_CONTROL_ALLOW_HEADERS,
    ACCESS_CONTROL_ALLOW_METHODS,
    ACCESS_CONTROL_ALLOW_ORIGIN,
    ACCESS_CONTROL_EXPOSE_HEADERS,
    ACCESS_CONTROL_MAX_AGE,
    ACCESS_CONTROL_REQUEST_HEADERS,
    ACCESS_CONTROL_REQUEST_METHOD,
    AGE,
    ALLOW,
    ALT_SVC,
    AUTHORIZATION,
    CACHE_CONTROL,
    CONNECTION,
    CONTENT_DISPOSITION,
    CONTENT_ENCODING,
    CONTENT_LANGUAGE,
    CONTENT_LENGTH,
    CONTENT_LOCATION,
    CONTENT_MD5,
    CONTENT_RANGE,
    CONTENT_SECURITY_POLICY,
    CONTENT_SECURITY_POLICY_REPORT_ONLY,
    CONTENT_TYPE,
    COOKIE,
    DNT,
    DATE,
    ETAG,
    EXPECT,
    EXPIRES,
    FORWARDED,
    FROM,
    HOST,
    IF_MATCH,
    IF_MODIFIED_SINCE,
    IF_NONE_MATCH,
    IF_RANGE,
    IF_UNMODIFIED_SINCE,
    LAST_MODIFIED,
    KEEP_ALIVE,
    LINK,
    LOCATION,
    MAX_FORWARDS,
    ORIGIN,
    PRAGMA,
    PROXY_AUTHENTICATE,
    PROXY_AUTHORIZATION,
    PUBLIC_KEY_PINS,
    PUBLIC_KEY_PINS_REPORT_ONLY,
    RANGE,
    REFERER,
    REFERRER_POLICY,
    REFRESH,
    RETRY_AFTER,
    SERVER,
    SET_COOKIE,
    STRICT_TRANSPORT_SECURITY,
    TE,
    TK,
    TRAILER,
    TRANSFER_ENCODING,
    TSV,
    USER_AGENT,
    UPGRADE,
    UPGRADE_INSECURE_REQUESTS,
    VARY,
    VIA,
    WARNING,
    WWW_AUTHENTICATE,
    X_CONTENT_TYPE_OPTIONS,
    X_DNS_PREFETCH_CONTROL,
    X_FRAME_OPTIONS,
    X_XSS_PROTECTION,
];
