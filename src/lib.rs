#![no_std]

pub mod gpio;
pub mod uart;
pub mod adc;

#[derive(Debug)]
pub enum Never {}