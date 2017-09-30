extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use std::default::Default;
use std::ops::Deref;
use std::thread;
use std::time::Duration;

use r2d2_redis::RedisConnectionManager;

fn main() {
    let config = Default::default();
    let manager = RedisConnectionManager::new("redis://localhost", Duration::from_secs(1)).unwrap();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    let mut handles = vec![];

    for _i in 0..10i32 {
        let pool = pool.clone();
        handles.push(thread::spawn(move || {
            let conn = pool.get().unwrap();
            let reply = redis::cmd("PING").query::<String>(conn.deref()).unwrap();
            // Alternatively, without deref():
            // let reply = redis::cmd("PING").query::<String>(&*conn).unwrap();
            assert_eq!("PONG", reply);
        }));
    }

    for h in handles {
        h.join().unwrap();
    }
}
