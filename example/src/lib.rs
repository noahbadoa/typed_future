#![no_implicit_prelude]
extern crate core;
extern crate std;
extern crate typed_future;
extern crate pin_project;

#[allow(unused)]
mod reference;
#[cfg(test)]
mod tests;
#[allow(unused)]
mod common;

// cargo +nightly miri test
