#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_halt as _;

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self, usart1}, 
    prelude::*, 
    serial::{Config, Serial}, 
    timer::Timer
};

use core::fmt::Write; //write! 매크로 사용
use nb::block;

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

    // GPIO 설정
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();

    //PA9 = TX, PA10 = RX
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx  = gpioa.pa10;

    // UART 설정
    let serial = Serial::new(
        dp.USART1,
        (pin_tx,pin_rx),
        &mut afio.mapr,
        Config::default()
            .baudrate(115_200.bps())
            .wordlength_8bits()
            .parity_none(),
        &clocks
    );
    let (mut tx, _rx) = serial.split();
    
    // SYST 타이머 기반 딜레이 생성
    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    loop {
        led.set_low();    // 켜기 (PC13은 active-low)
        delay.delay_ms(500u16);

        led.set_high();   // 끄기
        delay.delay_ms(500u16);
        
        
        writeln!(tx, "hello, Rust from STM32!").unwrap()
    }
}