extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use std::sync::Arc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use r2d2_redis::RedisConnectionManager;

#[test]
fn test_basic() {
    let manager = RedisConnectionManager::new("redis://localhost", Duration::from_secs(1)).unwrap();
    let config = r2d2::Config::builder().pool_size(2).build();
    let pool = Arc::new(r2d2::Pool::new(config, manager).unwrap());

    let (s1, r1) = mpsc::channel();
    let (s2, r2) = mpsc::channel();

    let pool1 = pool.clone();
    let t1 = thread::spawn(move || {
        let conn = pool1.get().unwrap();
        s1.send(()).unwrap();
        r2.recv().unwrap();
        drop(conn);
    });

    let pool2 = pool.clone();
    let t2 = thread::spawn(move || {
        let conn = pool2.get().unwrap();
        s2.send(()).unwrap();
        r1.recv().unwrap();
        drop(conn);
    });

    t1.join().unwrap();
    t2.join().unwrap();

    pool.get().unwrap();
}

#[test]
fn test_is_valid() {
    let manager = RedisConnectionManager::new("redis://localhost", Duration::from_secs(1)).unwrap();
    let config = r2d2::Config::builder()
        .pool_size(1)
        .test_on_check_out(true)
        .build();
    let pool = r2d2::Pool::new(config, manager).unwrap();

    pool.get().unwrap();
}
