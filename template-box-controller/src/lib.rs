#![no_std]

pub extern crate stm32f4xx_hal as hal;

pub mod error;
pub mod logger;
pub mod net;
pub mod params;
pub mod sync;
pub mod sys_clock;
pub mod time;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
