//! Core support library for ironos bare-metal kernel.

#![no_std]
#![warn(missing_docs)]

pub mod allocator;
pub mod ringbuf;
pub mod sv39;
