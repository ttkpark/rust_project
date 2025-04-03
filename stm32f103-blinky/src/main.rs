#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use stm32f1xx_hal::{
    pac,
    prelude::*, // GPIO 설정
    timer::Timer, // delay
    gpio::gpioc::PC13,
    gpio::{Output, PushPull},
    time::U32Ext
};

#[entry]
fn main() -> ! {
    // 디바이스와 코어 주변장치
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // 클럭 구성 (외부 8MHz 크리스탈 → 72MHz 시스템 클럭)
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();

    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())         // ✅ .mhz() → .MHz() 로 변경
        .sysclk(72.MHz())
        .pclk1(36.MHz())
        .freeze(&mut flash.acr);

    // GPIOC에서 PC13 (LED 핀) 설정
    let mut gpioc = dp.GPIOC.split();
    let mut led: PC13<Output<PushPull>> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // SYST 타이머 기반 딜레이 생성
    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    loop {
        led.set_low();    // 켜기 (PC13은 active-low)
        delay.delay_ms(500u16);

        led.set_high();   // 끄기
        delay.delay_ms(500u16);
    }
}