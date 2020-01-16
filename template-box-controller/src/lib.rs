#![no_std]

pub extern crate stm32f4xx_hal as hal;

use core::cell::Cell;
use cortex_m::interrupt::Mutex;
use heapless::mpmc::Q32;

// TODO - use a prelude?

pub mod error;
pub mod logger;
pub mod net;
pub mod params;
pub mod sync;
pub mod sys_clock;
pub mod time;

static PARAM_EVENT_Q: Q32<crate::params::Event> = Q32::new();
static SYSTEM_MILLIS: Mutex<Cell<u64>> = Mutex::new(Cell::new(0));
