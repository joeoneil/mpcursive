#![feature(const_maybe_uninit_zeroed)]

use std::mem::MaybeUninit;

use cursive::Cursive;

static mut SIV: MaybeUninit<Cursive> = MaybeUninit::zeroed();

pub mod mpd_util;
pub mod view;

pub fn init() {
    unsafe {
        SIV.write(Cursive::new());
    }
}

pub fn global_cursive() -> &'static mut Cursive {
    unsafe { SIV.as_mut_ptr().as_mut().unwrap() }
}
