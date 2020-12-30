#![warn(clippy::all, rust_2018_idioms)]
#![deny(missing_docs, unused, nonstandard_style, future_incompatible)]
#![feature(test)]

//! # Compiler (System-oriented Programming)
//!
//! A simple compiler reimplementation inspired by a former study project.

mod buffer;

pub use crate::buffer::CharBuffer;
