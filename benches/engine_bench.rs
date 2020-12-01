#[macro_use]
extern crate criterion;

use criterion::{BenchmarkId, Criterion};
use kvs::{KvStore, KvsEngine};
use rand::prelude::*;
use tempfile::TempDir;
use walkdir::WalkDir;

const SCALE: [u32; 7] = [4, 6, 8, 10, 12, 14, 16];

fn dir_size(dir: &TempDir) -> u64 {
    let entries = WalkDir::new(dir.path()).into_iter();
    let len: walkdir::Result<u64> = entries
        .map(|res| {
            res.and_then(|entry| entry.metadata())
                .map(|metadata| metadata.len())
        })
        .sum();
    len.expect("fail to get directory size")
}

pub fn set_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("set");

    for i in SCALE.iter() {
        let (mut size, mut cnt) = (0, 0);
        group.bench_with_input(BenchmarkId::new("kvs", i), i, |b, n| {
            b.iter(|| {
                let dir = TempDir::new().unwrap();
                let path = dir.path();
                let kvs = KvStore::open(path).unwrap();

                let value = "value".to_string();
                for i in 0..(1 << n) {
                    let key = format!("key{}", i);
                    kvs.set(key, value.clone()).unwrap();
                }

                size += dir_size(&dir);
                cnt += 1;
            })
        });
        println!("kvs[{}] dir size: {}", i, (size as f64) / (cnt as f64));

        let (mut size, mut cnt) = (0, 0);
        group.bench_with_input(BenchmarkId::new("sled", i), i, |b, n| {
            b.iter(|| {
                let dir = TempDir::new().unwrap();
                let path = dir.path();
                let sled = sled::open(path).unwrap();

                let value = "value".to_string();
                for i in 0..(1 << n) {
                    let key = format!("key{}", i);
                    sled.insert(key, value.clone().into_bytes()).unwrap();
                }

                size += dir_size(&dir);
                cnt += 1;
            })
        });
        println!("sled[{}] dir size: {}", i, (size as f64) / (cnt as f64));
    }
}

pub fn full_bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("full");

    for scale in SCALE.iter() {
        {
            let (mut size, mut cnt) = (0, 0);
            group.bench_with_input(BenchmarkId::new("kvs", scale), scale, |b, n| {
                b.iter(|| {
                    let dir = TempDir::new().unwrap();
                    let path = dir.path();

                    let kvs = KvStore::open(path).unwrap();
                    for i in 0..(1 << scale) {
                        let key = format!("key{}", i);
                        let value = format!("value{}", i);

                        kvs.set(key, value).unwrap();
                    }

                    let mut rng = rand::thread_rng();
                    for _i in 0..(1 << n) {
                        let key = format!("key{}", rng.gen_range(0, 1 << n));
                        kvs.get(key).unwrap();
                    }

                    let mut rng = rand::thread_rng();
                    for _i in 0..(1 << n) {
                        let key = format!("key{}", rng.gen_range(0, 1 << n));
                        match kvs.remove(key) {
                            _ => (),
                        }
                    }

                    size += dir_size(&dir);
                    cnt += 1;
                })
            });
            println!("kvs[{}] dir size: {}", scale, (size as f64) / (cnt as f64));
        }

        {
            let (mut size, mut cnt) = (0, 0);
            group.bench_with_input(BenchmarkId::new("sled", scale), scale, |b, n| {
                b.iter(|| {
                    let dir = TempDir::new().unwrap();
                    let path = dir.path();

                    let sled = sled::open(path).unwrap();
                    for i in 0..(1 << scale) {
                        let key = format!("key{}", i);
                        let value = format!("value{}", i);

                        sled.insert(key, value.into_bytes()).unwrap();
                    }

                    let mut rng = rand::thread_rng();
                    for _i in 0..(1 << n) {
                        let key = format!("key{}", rng.gen_range(0, 1 << n));
                        sled.get(key).unwrap();
                    }

                    let mut rng = rand::thread_rng();
                    for _i in 0..(1 << n) {
                        let key = format!("key{}", rng.gen_range(0, 1 << n));
                        match sled.remove(key) {
                            _ => (),
                        }
                    }

                    size += dir_size(&dir);
                    cnt += 1;
                })
            });
            println!("sled[{}] dir size: {}", scale, (size as f64) / (cnt as f64));
        }
    }
}

criterion_group!(benches, set_bench, full_bench);
criterion_main!(benches);
