#![feature(map_try_insert)]

use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::{alloc, collections::HashMap, mem};

use memcheck::Memcheck;

#[global_allocator]
static ALLOCATOR: Memcheck<alloc::System> = Memcheck::new(alloc::System);

#[test]
fn smoke() {
	let rng = &mut SmallRng::seed_from_u64(0);
	let iters = if cfg!(miri) { 10_000 } else { 100_000_000 };
	let mut a: Vec<u8> = Vec::new();
	let mut b: Vec<u128> = Vec::new();
	let mut c: HashMap<u64, u64> = HashMap::new();
	for i in 0..iters {
		if i % (iters / 100) == 0 {
			println!("{i}");
		}
		match rng.random_range(0..3) {
			0 => {
				if rng.random_range(0..10_000) != 0 {
					a.push(rng.random());
				} else {
					_ = mem::take(&mut a);
				}
			}
			1 => {
				if rng.random_range(0..10_000) != 0 {
					b.push(rng.random());
				} else {
					_ = mem::take(&mut b);
				}
			}
			2 => {
				if rng.random_range(0..10_000) != 0 {
					_ = c.try_insert(rng.random(), rng.random()).unwrap();
				} else {
					_ = mem::take(&mut c);
				}
			}
			_ => unreachable!(),
		}
	}
}
