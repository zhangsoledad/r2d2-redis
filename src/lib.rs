//! Redis support for the `r2d2` connection pool.
#![doc(html_root_url = "https://sorccu.github.io/r2d2-redis/doc/v0.6.0")]
#![warn(missing_docs)]
extern crate r2d2;
extern crate redis;

use std::error;
use std::error::Error as _StdError;
use std::fmt;
use std::time::Duration;

/// A unified enum of errors returned by redis::Client
#[derive(Debug)]
pub enum Error {
    /// A redis::RedisError
    Other(redis::RedisError),
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self.cause() {
            Some(cause) => write!(fmt, "{}: {}", self.description(), cause),
            None => write!(fmt, "{}", self.description()),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Other(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            Error::Other(ref err) => err.cause(),
        }
    }
}

/// An `r2d2::ConnectionManager` for `redis::Client`s.
///
/// ## Example
///

/// ```
/// extern crate r2d2;
/// extern crate r2d2_redis;
/// extern crate redis;
///
/// use std::default::Default;
/// use std::ops::Deref;
/// use std::thread;
/// use std::time::Duration;
///
/// use r2d2_redis::RedisConnectionManager;
///
/// fn main() {
///     let manager = RedisConnectionManager::new("redis://localhost", Duration::from_secs(1)).unwrap();
///     let pool = r2d2::Pool::builder()
///         .max_size(15)
///         .build(manager)
///         .unwrap();
///
///     let mut handles = vec![];
///
///     for _i in 0..10i32 {
///         let pool = pool.clone();
///         handles.push(thread::spawn(move || {
///             let conn = pool.get().unwrap();
///             let reply = redis::cmd("PING").query::<String>(conn.deref()).unwrap();
///             // Alternatively, without deref():
///             // let reply = redis::cmd("PING").query::<String>(&*conn).unwrap();
///             assert_eq!("PONG", reply);
///         }));
///     }
///
///     for h in handles {
///         h.join().unwrap();
///     }
/// }
/// ```
#[derive(Debug)]
pub struct RedisConnectionManager {
    connection_info: redis::ConnectionInfo,
    timeout: Duration,
}

impl RedisConnectionManager {
    /// Creates a new `RedisConnectionManager`.
    ///
    /// See `redis::Client::open` for a description of the parameter
    /// types.
    pub fn new<T: redis::IntoConnectionInfo>(
        params: T,
        timeout: Duration,
    ) -> Result<RedisConnectionManager, redis::RedisError> {
        Ok(RedisConnectionManager {
            connection_info: try!(params.into_connection_info()),
            timeout: timeout,
        })
    }
}

impl r2d2::ManageConnection for RedisConnectionManager {
    type Connection = redis::Connection;
    type Error = Error;

    fn connect(&self) -> Result<redis::Connection, Error> {
        match redis::Client::open(self.connection_info.clone()) {
            Ok(client) => match client.get_connection() {
                Ok(conn) => {
                    conn.set_write_timeout(Some(self.timeout))
                        .map_err(Error::Other)?;
                    conn.set_read_timeout(Some(self.timeout))
                        .map_err(Error::Other)?;
                    Ok(conn)
                }
                Err(e) => Err(Error::Other(e)),
            },
            Err(err) => Err(Error::Other(err)),
        }
    }

    fn is_valid(&self, conn: &mut redis::Connection) -> Result<(), Error> {
        redis::cmd("PING").query(conn).map_err(Error::Other)
    }

    fn has_broken(&self, _conn: &mut redis::Connection) -> bool {
        false
    }
}
