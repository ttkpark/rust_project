# Day 6
- day 6은 UART 통신하기가 주요 목표다.
- <img src="../res/화면 캡처 2025-04-07 232555.png" >

- 매우 전송이 잘 된다.

## 코드 리뷰
```rust
....

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self, usart1}, 
    prelude::*, 
    serial::{Config, Serial},  // Serial를 위한 implementation
    timer::Timer
};

use core::fmt::Write; //write! 매크로 사용
use nb::block; // unblocked 함수를 blocked 함수처럼 사용

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

    //PA9 = TX(출력), PA10 = RX(입력) 설정정
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx  = gpioa.pa10;

    // UART 설정
    let serial = Serial::new(
        dp.USART1,       // 통신 레지스터 인스턴스
        (pin_tx,pin_rx), // 핀 2개
        &mut afio.mapr,  // AFIO가 무엇인지 아직 모름
        Config::default() // 설정 객체
            .baudrate(115_200.bps()) // 115200bps의 샘플 속도로
            .wordlength_8bits()      // 전송 단위는 8bit
            .parity_none(),          // parity 설정은 없다.
        &clocks           // 시간 객체 (baud pres 시간 계산 시 필요할 것으로 추정정)
    );
    let (mut tx, _rx) = serial.split(); // rx. tx 객체로 나눔. _rx처럼 앞에 언더바로 시작하면 컴파일에서는 사용하지 않는 변수라 못 박는다.
    
    // SYST 타이머 기반 딜레이 생성
    let mut delay = Timer::syst(cp.SYST, &clocks).delay();

    loop {
        led.set_low();    // 켜기 (PC13은 active-low)
        delay.delay_ms(500u16);

        led.set_high();   // 끄기
        delay.delay_ms(500u16);
        
        
        writeln!(tx, "hello, Rust from STM32!").unwrap() // 출력 함수
    }
}
```