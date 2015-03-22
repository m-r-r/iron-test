#![deny(missing_docs)]
#![deny(warnings)]
#![feature(core, std_misc, path_ext)]

//! A set of constructors for mocking Iron objects.

extern crate iron;
extern crate hyper;
extern crate url;
extern crate uuid;

#[macro_use]
extern crate log;

pub use project_builder::ProjectBuilder;

mod project_builder;


/// Contains tooling for mocking various Iron objects.
pub mod mock {
    use std::{io, net};
    use std::io::Cursor;
    use hyper::net::NetworkStream;

    /// A network string implementation for in-memory buffers
    pub struct MockStream(io::Cursor<Vec<u8>>);

    impl Clone for MockStream {
        fn clone(&self) -> MockStream {
            MockStream(Cursor::new(self.0.get_ref().clone()))
        }
    }

    impl MockStream {
        /// Create a new MockStream wrapping the provided byte string.
        pub fn new(input: &[u8]) -> MockStream {
            MockStream(Cursor::new(input.to_vec()))
        }
    }

    impl io::Read for MockStream {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0.read(buf)
        }
    }

    impl io::Write for MockStream {
        fn write(&mut self, msg: &[u8]) -> io::Result<usize> {
            Ok(msg.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl NetworkStream for MockStream {
        fn peer_addr(&mut self) -> io::Result<net::SocketAddr> {
            Ok("[::1]:2468".parse().unwrap())
        }
    }

    /// Contains constructors for mocking Iron Requests.
    pub mod request {
        use iron::{Request, TypeMap, Headers, Url};
        use iron::request::Body;
        use iron::{method, headers};

        use hyper::http::HttpReader;
        use hyper::net::NetworkStream;

        use std::io::BufReader;
        use std::net::SocketAddr;

        /// Create a new mock Request with the given method, url, and data.
        pub fn new<'a, 'b: 'a>(method: method::Method, path: Url,
                               buffer: &'a mut BufReader<&'b mut NetworkStream>) -> Request<'a, 'b> {
            let reader = HttpReader::EofReader(buffer);
            let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();

            let mut headers = Headers::new();
            let host = Url::parse("http://127.0.0.1:3000").unwrap()
                .into_generic_url()
                .serialize_host().unwrap();

            headers.set(headers::Host {
                hostname: host,
                port: Some(3000),
            });

            headers.set(headers::UserAgent("iron-test".to_string()));

            Request {
                method: method,
                url: path,
                body: Body::new(reader),
                local_addr: addr.clone(),
                remote_addr: addr,
                headers: headers,
                extensions: TypeMap::new()
            }
        }
    }
}

#[cfg(test)]
mod test {
    mod request {
        use super::super::mock::request;
        use super::super::mock::MockStream;
        use iron::method;
        use iron::Url;
        use std::io::{Read,BufReader};
        use hyper::net::NetworkStream;

        #[test] fn test_request() {
            let ref mut stream = MockStream::new("Hello Google!".as_bytes());
            let ref mut buffer = BufReader::new(stream as &mut NetworkStream);
            let mut req = request::new(method::Get, Url::parse("http://localhost:3000").unwrap(), buffer);
            assert_eq!(req.method, method::Get);
            assert_eq!(format!("{}", req.url).as_slice(), "http://localhost:3000/");

            let mut body_buf = Vec::new();
            req.body.read_to_end(&mut body_buf).ok().unwrap();
            assert_eq!(&*body_buf, b"Hello Google!");
        }
    }
}

