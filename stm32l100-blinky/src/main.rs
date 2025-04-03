#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use stm32l0xx_hal::{
    prelude::*,
    pac,
    delay::Delay,
};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.freeze(stm32l0xx_hal::rcc::Config::hsi16());
    let mut gpioa = dp.GPIOA.split(&mut rcc);
    let mut gpioc = dp.GPIOC.split(&mut rcc);
    let mut ledg = gpioc.pc9.into_push_pull_output();
    let mut ledb = gpioc.pc8.into_push_pull_output();
    let mut delay = Delay::new(cp.SYST, &rcc);

    loop {
        ledg.set_high().unwrap();
        ledb.set_low().unwrap();
        delay.delay_ms(500u32);
        ledg.set_low().unwrap();
        ledb.set_high().unwrap();
        delay.delay_ms(500u32);
    }
}