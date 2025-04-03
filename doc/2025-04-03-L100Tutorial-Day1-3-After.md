# Rust 임베디드 생태계
1. Core crates
2. HAL
3. RTOS-like 프레임워크
4. 빌드도구


# 🪟 Windows 환경용: Rust + 임베디드 개발 설치 가이드
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
rustup target add thumbv6m-none-eabi
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
```toml
[package]
# ...
build = "build.rs"

[dependencies]
# 기존 내용

[build-dependencies]
cc = "1.0"
```
- `build.rs` 작성
```rust
fn main() {
    println!("cargo:rerun-if-changed=memory.x");
    println!("cargo:rustc-link-search=.");
}
```
4. `Cargo.toml`업데이트하기기
```toml
[package]
name = "stm32l100-blinky"
version = "0.1.0"
edition = "2021"
build = "build.rs"

[dependencies]
cortex-m = "0.6"
cortex-m-rt = "0.6"
stm32l0xx-hal = "0.9.0"
panic-halt = "0.2.0"

[profile.dev]
codegen-units = 1
debug = true
lto = false
opt-level = "s"

[profile.release]
codegen-units = 1
debug = false
lto = true
opt-level = "z"

[build-dependencies]
cc = "1.0"
```
버전 옆에 체크가 뜨는데 그거 따라가야 할까

5. blink 코드 작성
```rust
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
```

에러 1. Must select exactly one package for linker script generation!
  Choices: 'stm32l0x1' or 'stm32l0x2' or 'stm32l0x3'
  Alternatively, pick the mcu-feature that matches your MCU, for example 'mcu-STM32L071KBTx'

   Must select exactly one flash size for linker script generation!
  Choices: 'flash-8', 'flash-16', 'flash-32', 'flash-64', 'flash-128' or 'flash-192'
   Alternatively, pick the mcu-feature that matches your MCU, for example 'mcu-STM32L071KBTx'

   인터넷에 STM32L100RCTx 쳐보니 stm32 arm keil 홈페이지 들어가서보면 Flash 256Kib, SRAM 16KiB라고 나와있다.
   memory.x에서 256K, 16K라고 쓰자.


# 에러사항
- delay 라이브러리 및 Hz 함수를 찾는데 애를 먹었다.
- 빌드했는데 elf 파일 크기가 약 700바이트이고, bin 크기가 0byte다. 


- gitignore에 대해 알아보았다.
https://nesoy.github.io/blog/Git-ignore
