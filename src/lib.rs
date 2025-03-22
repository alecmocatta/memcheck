#![feature(ptr_as_ref_unchecked)]

use static_assertions::assert_eq_align;
use std::{
	alloc::{GlobalAlloc, Layout},
	fmt::{self, Debug},
	ptr,
};

#[derive(Debug)]
pub struct Memcheck<H> {
	allocator: H,
}

impl<H> Memcheck<H> {
	#[inline(always)]
	pub const fn new(allocator: H) -> Self {
		Self { allocator }
	}
	#[inline(always)]
	fn enhance_layout(l: Layout) -> Layout {
		let new_size = l.size() + size_of::<Check>();
		Layout::from_size_align(new_size, l.align()).expect("memcheck: alloc too big")
	}
}

unsafe impl<H> GlobalAlloc for Memcheck<H>
where
	H: GlobalAlloc,
{
	unsafe fn alloc(&self, l: Layout) -> *mut u8 {
		// writeln!(StderrRaw, "alloc {l:?}").unwrap();
		let ret = self.allocator.alloc(Self::enhance_layout(l));
		if !ret.is_null() {
			let check = Check::new(l.size(), l.align(), 1);
			ptr::write(ret.add(l.size()).cast::<Check>(), check);
		}
		ret
	}
	unsafe fn dealloc(&self, ptr: *mut u8, l: Layout) {
		// writeln!(StderrRaw, "dealloc {l:?}").unwrap();
		let check = ptr.add(l.size()).cast::<Check>().as_mut_unchecked();
		assert_eq!((l.size(), l.align(), 1), (check.size(), check.align(), check.live()));
		check.set_live(0);
		self.allocator.dealloc(ptr, Self::enhance_layout(l));
	}
	unsafe fn alloc_zeroed(&self, l: Layout) -> *mut u8 {
		// writeln!(StderrRaw, "alloc_zeroed {l:?}").unwrap();
		let ret = self.allocator.alloc_zeroed(Self::enhance_layout(l));
		if !ret.is_null() {
			let check = Check::new(l.size(), l.align(), 1);
			ptr::write(ret.add(l.size()).cast::<Check>(), check);
		}
		ret
	}
	unsafe fn realloc(&self, ptr: *mut u8, l: Layout, new_size: usize) -> *mut u8 {
		// writeln!(StderrRaw, "realloc {l:?}").unwrap();
		let check = ptr.add(l.size()).cast::<Check>().as_mut_unchecked();
		assert_eq!((l.size(), l.align(), 1), (check.size(), check.align(), check.live()));
		check.set_live(0);
		let new_layout = Layout::from_size_align(new_size, l.align()).unwrap();
		let ret = self.allocator.realloc(
			ptr,
			Self::enhance_layout(l),
			Self::enhance_layout(new_layout).size(),
		);
		if !ret.is_null() {
			let check = Check::new(new_layout.size(), new_layout.align(), 1);
			ptr::write(ret.add(new_layout.size()).cast::<Check>(), check);
		}
		ret
	}
}

#[derive(Clone, Copy)]
#[repr(transparent)]
struct Check([u8; size_of::<usize>() + size_of::<u8>() + size_of::<u8>()]);
assert_eq_align!(u8, Check);
impl Check {
	#[inline(always)]
	fn new(size: usize, align: usize, live: u8) -> Self {
		let mut ret = Self([0; size_of::<usize>() + size_of::<u8>() + size_of::<u8>()]);
		ret.set_size(size);
		ret.set_align(align);
		ret.set_live(live);
		ret
	}
	#[inline(always)]
	fn size(&self) -> usize {
		usize::from_ne_bytes(self.0[0..size_of::<usize>()].try_into().unwrap())
	}
	#[inline(always)]
	fn set_size(&mut self, size: usize) {
		self.0[0..size_of::<usize>()].copy_from_slice(&size.to_ne_bytes());
	}
	#[inline(always)]
	fn align(&self) -> usize {
		1 << self.0[size_of::<usize>()]
	}
	#[inline(always)]
	fn set_align(&mut self, align: usize) {
		assert!(align.is_power_of_two());
		self.0[size_of::<usize>()] = u8::try_from(align.ilog2()).unwrap();
	}
	#[inline(always)]
	fn live(&self) -> u8 {
		self.0[size_of::<usize>() + size_of::<u8>()]
	}
	#[inline(always)]
	fn set_live(&mut self, live: u8) {
		self.0[size_of::<usize>() + size_of::<u8>()] = live;
	}
}
impl Debug for Check {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Check")
			.field("size", &self.size())
			.field("align", &self.align())
			.field("live", &self.live())
			.finish()
	}
}
