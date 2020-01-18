use core::intrinsics;
use core::panic::PanicInfo;
use lib::hal::stm32::GPIOB;
use log::error;

#[panic_handler]
fn panic_handler(info: &PanicInfo) -> ! {
    error!("{}", info);
    // Turn all LEDs on, PB0, PB7, PB14
    for bit_index in [0_u32, 7_u32, 14_u32].iter() {
        unsafe {
            // Into push pull output
            let offset: u32 = 2 * bit_index;
            &(*GPIOB::ptr())
                .pupdr
                .modify(|r, w| w.bits((r.bits() & !(0b11_u32 << offset)) | (0b00 << offset)));
            &(*GPIOB::ptr())
                .otyper
                .modify(|r, w| w.bits(r.bits() & !(0b1_u32 << bit_index)));
            &(*GPIOB::ptr())
                .moder
                .modify(|r, w| w.bits((r.bits() & !(0b11_u32 << offset)) | (0b01_u32 << offset)));

            // Set high
            (*GPIOB::ptr()).bsrr.write(|w| w.bits(1_u32 << bit_index));
        }
    }
    unsafe { intrinsics::abort() }
}
