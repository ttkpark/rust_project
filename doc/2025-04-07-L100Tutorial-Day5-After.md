# 코드 보기
- 메모리와 링커 구조

- 코드 분석
```rust
#![no_std] // OS가 없으므로 std lib 사용 불가
#![no_main] // OS가 없으므로 main 함수를 호출해 주지 않음. 직접 #[entry]를 통해 진입점을 명시함.

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f1xx_hal::{};

#[entry] // 진입점 직접 명시, 이 함수는 무한 루프타입(-> !)이어야 하고, 이 함수는 Reset Vector로 등록됨.
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

// 첫 빌드 결과 요약약
//   text    data     bss     dec     hex filename
//    800       0       4     804     324 target/thumbv7m-none-eabi/release/stm32f103-blinky

// .text, .data, .bss, .stack
// .text = 프로그램 코드량, FLASH에 저장되는 데이터
// .data = 초기화된 전역 변수(RAM)
// .bss = 초기화되지 않은 전역 변수(RAM)
// .stack, .heap = 런타임 스택/힙 (지역 변수 등)
// dec = 전체 크기(10진수), hex = 전체 크기(16진수) : text+data+bss
```
