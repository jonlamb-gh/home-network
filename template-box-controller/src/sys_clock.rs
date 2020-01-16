use crate::time::Instant;
use crate::SYSTEM_MILLIS;
use cortex_m::interrupt::CriticalSection;
use cortex_m::peripheral::syst::SystClkSource;
use hal::rcc::Clocks;
use hal::stm32::SYST;
use log::debug;

pub fn start(mut syst: SYST, clocks: Clocks) {
    debug!("Enable SystemClock freq {} Hz", clocks.hclk().0);

    // Generate an interrupt once a millisecond
    syst.set_clock_source(SystClkSource::External);
    syst.set_reload(clocks.hclk().0 / 8_000);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    // So the SYST can't be stopped or reset
    drop(syst);
}

pub fn increment(cs: &CriticalSection) {
    let cell = SYSTEM_MILLIS.borrow(cs);
    let t = cell.get();
    cell.replace(t.wrapping_add(1));
}

/// Time elapsed since `SystemClock` was started
pub fn system_time() -> Instant {
    Instant::from_millis(system_millis())
}

#[cfg(not(test))]
pub fn system_millis() -> u64 {
    cortex_m::interrupt::free(|cs| SYSTEM_MILLIS.borrow(cs).get())
}

#[cfg(test)]
pub fn system_millis() -> u64 {
    0
}
