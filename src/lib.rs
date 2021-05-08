#![no_std]

pub mod gpio;
pub mod uart;
pub mod adc;
pub mod time;

#[derive(Debug)]
pub enum Never {}