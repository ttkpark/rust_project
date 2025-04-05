# Rust 임베디드 생태계
1. Core crates
2. HAL
3. RTOS-like 프레임워크
4. 빌드도구

# Rust + 임베디드 개발 설치 과정정
- 설치 완료 조건 : 아래 명령어가 잘 싱행되어야 함.
```powershell
  rustup --version
  cargo --version
```
- 설치 과정
1. https://rustup.rs 접속
2. `rustup-init.exe` 설치, 이어서 뜨는 커맨드 창에 1 입력 (기본 설치)
 - 리눅스 서브시스템의 경우 아래 명령어 입력력
```shell
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
3. unistall 방법은은
```shell
  rustup self uninstall
```
4. 명령어 실행이 안되면?
  `C:\Users\username\.cargo\bin`에 rust가 깔렸는지 확인하기
  환경변수 PATH에 `C:\Users\username\.cargo\bin`가 등록되었는지 확인하기
  난 VS Code 터미널에선 바로 적용이 안되서 안떴다. 코드를 다시 껐다 켜보기.

5. STM32 Cortex-M0+ 타겟 추가 및 ARM 툴체인 설치
```shell
# Cortex-M0+ 코어 버전의 툴체인 -> thumbv6m-none-eabi
rustup target add thumbv6m-none-eabi

# gcc 버전 체크
arm-none-eabi-gcc --version
```
  1) 아래 주소에서 Windows 용 arm-none-eabi 툴체인 다운로드:
    https://developer.arm.com/downloads/-/gnu-rm

  2) 설치 후 환경 변수에 bin 경로 추가:
    예: C:\Program Files (x86)\GNU Arm Embedded Toolchain\10 2021.10\bin

6. VS Code rust Extension 설치

| 확장 이름 | 설명 |
|-----------|------|
| rust-analyzer | Rust 코드 자동 완성, 인텔리센스 |
| ~~crates~~ -> Dependi | `Cargo.toml` 의존성 자동 보기 |
| ~~Better TOML~~ -> Even Better TOML | `.toml` 문법 강조 |
| Cortex-Debug | STM32 디버깅 지원 (OpenOCD 또는 ST-Link 사용 시) |

# Day 3 : STM32L0 소개 & CubeMX 활용
- 핀맵 활용성

| 기능 | 핀 |
| ---- | ---- |
| LED Green | PC9 |
| LED Blue | PC8 |
| USER BUTTON | PA0 |
- 링크는 `/stm32l100-rust-ref/`으로 만들기.

# Day 4 : 첫 Rust 프로젝트 (Cargo + Blink 예제)
- Cargo 프로젝트 만들기
```shell
cargo new stm32l100-blinky --bin
cd stm32l100-blinky
```
- `.cargo/config.toml` 작성
```toml
[build]
target = "thumbv6m-none-eabi"

[target.thumbv6m-none-eabi]
runner = "arm-none-eabi-gdb"

[unstable]
build-std = ["core"]
```
3. 링커 스크립트 작성성
- 프로젝트 루트에 `memory.x` 작성
```
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 32K
  RAM   : ORIGIN = 0x20000000, LENGTH = 4K
}
```
- `Cargo.toml`에 `memory.x` 인식시키기
- `build.rs` 작성
4. `Cargo.toml`업데이트하기
5. blink 코드 작성

- 에러 1. Must select exactly one package for linker script generation!
  Choices: 'stm32l0x1' or 'stm32l0x2' or 'stm32l0x3'
  Alternatively, pick the mcu-feature that matches your MCU, for example 'mcu-STM32L071KBTx'

   Must select exactly one flash size for linker script generation!
  Choices: 'flash-8', 'flash-16', 'flash-32', 'flash-64', 'flash-128' or 'flash-192'
   Alternatively, pick the mcu-feature that matches your MCU, for example 'mcu-STM32L071KBTx'

- 인터넷에 STM32L100RCTx 쳐보니 stm32 arm keil 홈페이지 들어가서보면 Flash 256Kib, SRAM 16KiB라고 나와있다.
- memory.x에서 256K, 16K라고 쓰자.

# 오류 발생
 - STM32L100RCTx 칩은 처음에 ChatGPT가 생각한 Cortex-M0+가 아니었고 Cortex-M3였다. 그리고 Cortex-M3는 hal 기능을 제공하는 라이브러리인 `stm32l1xx-hal`이 없다. 따라서 HAL(High Abstraction Layer)의 강력한 기능을 사용하지 못하고 하드웨어 레지스터를 직접 제어해야함.(`stm32l1`이라는 PAC(Peripheral Access Crate) 사용)
 - 칩셋을 바꾸자. STM32F103은 지원한다. Cortex-M3이므로 thumbv7m-none-eabi 툴셋을 사용하고, HAL 라이브러리는 `stm32f1xx-hal`을 사용한다.

- gitignore에 대해 알아보았다.
https://nesoy.github.io/blog/Git-ignore


# 다시 stm32f103으로 만들었다
1. Rust Target 설정(Coretex-M3는 thumbv7m-none-eabi), 프로젝트 생성성
```bash
rustup target add thumbv7m-none-eabi 

cargo new stm32f103-blinky
cd stm32f103-blinky
```

3. `.cargo/config.toml` 생성:
```toml
[build]
target = "thumbv7m-none-eabi"

#[target.thumbv7m-none-eabi]
#runner = ""
# manual binary update through STM32Programmer 
rustflags = [ "-C", "link-arg=-Tlink.x"]

[unstable]
build-std = ["core"]
```

4. `Cargo.toml`업데이트하기
```toml
[package]
name = "stm32f103-blinky"
version = "0.1.0"
edition = "2024"
build = "build.rs"

[dependencies]
cortex-m = "0.7"
cortex-m-rt = "0.7"
panic-halt = "1.0.0"

[dependencies.stm32f1xx-hal]
version = "0.10"
features = ["stm32f103", "rt", "medium"]

[build-dependencies]
cc = "1.2.18"

[profile.dev]
opt-level = "s"
#debug = true
lto = false
codegen-units = 1

[profile.release]
opt-level = "z"
debug = false
lto = true
codegen-units = 1
```


5. blink 코드 작성
```rust
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
```

6. 빌드 스크립트
```bash
# 빌드 명령어
cargo build --release

# 빌드 지우기 명령어
cargo clean

# 빌드 결과 프로그램 사이즈 측정
arm-none-eabi-size target/thumbv7m-none-eabi/release/stm32f103-blinky

# 빌드 결과 파일을 binary 파일로 해석
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/stm32f103-blinky firmware.bin
```

# 에러사항
- delay 라이브러리 및 Hz 함수를 찾는데 애를 먹었다.
- 빌드했는데 elf 파일 크기가 약 700바이트이고, bin 크기가 0byte다. (arm-none-eabi-size target/.../stm32f103-blinky 결과 모든 수치가 0바이트트)
  - ChatGPT에서 `.cargo/config.toml` 파일을 쓸 때 ```rustflags = [ "-C", "link-arg=-Tlink.x"]``` 내용을 빼먹었는데, 직접 (stm32f1 프로젝트)[https://jonathanklimt.de/electronics/programming/embedded-rust/rust-on-stm32-2/]를 찾아본 결과 이걸 넣음으로써 0바이트 오류를 해결할 수 있었다. (아오 ChatGPT 제대로 안찾아보나)

- STM32 시리즈별로 제공되는 rust hal library
1. `stm32f0xx-hal` - For STM32F0 series
2. `stm32f1xx-hal` - For STM32F1 series
3. `stm32f3xx-hal` - For STM32F3 series
4. `stm32f4xx-hal` - For STM32F4 series
5. `stm32f7xx-hal` - For STM32F7 series
6. `stm32g0xx-hal` - For STM32G0 series
7. `stm32g4xx-hal` - For STM32G4 series
8. `stm32h7xx-hal` - For STM32H7 series
9. `stm32l0xx-hal` - For STM32L0 series
10. `stm32l1xx-hal` - For STM32L1 series
11. `stm32l4xx-hal` - For STM32L4 series
12. `stm32wlxx-hal` - For STM32WL series
