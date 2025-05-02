[![Embedded Rust: running an embassy example on a STM32F411 microcontroller - YouTube](https://tse3.mm.bing.net/th?id=OIP.ivw878I7TnWkp1_PORwOJQHaEK&pid=Api)](https://www.youtube.com/watch?v=x_EuOvLXp_U)

좋아! 이제 **Day 11**을 진행해보자.  
오늘은 **Embassy 기반 Task 구조 실습**을 통해, `#[embassy_executor::main]` 구조를 사용하여 작업을 나누고, `async` 태스크를 활용한 LED 제어와 버튼 입력 처리까지 실습해볼 거야.

---

## 📘 Day 11: Embassy 기반 Task 구조 실습

### 🎯 오늘의 목표

- `#[embassy_executor::main]` 매크로를 사용하여 async 환경 설정
- `async fn` 태스크 생성 및 실행
- `embassy_time::Timer`를 활용한 비동기 딜레이 구현
- 버튼 입력을 비동기적으로 처리하여 LED 제어

---

## 🛠️ 1. 환경 설정

### 🔧 Cargo.toml 설정

```toml
[dependencies]
embassy-stm32 = { version = "0.1.0", features = ["stm32f103c8", "time-driver-any", "exti"] }
embassy-executor = { version = "0.3", features = ["arch-cortex-m"] }
embassy-time = { version = "0.1.0" }
cortex-m = "0.7"
cortex-m-rt = "0.7"
defmt = "0.3"
defmt-rtt = "0.3"
panic-probe = "0.3"
```

### 🔧 `.cargo/config.toml` 설정

```toml
[build]
target = "thumbv7m-none-eabi"

[unstable]
build-std = ["core", "compiler_builtins"]
```

### 🔧 `memory.x` 설정

```ld
MEMORY
{
  FLASH : ORIGIN = 0x08000000, LENGTH = 64K
  RAM   : ORIGIN = 0x20000000, LENGTH = 20K
}
```

---

## 💡 2. 기본 구조: `#[embassy_executor::main]`

Embassy에서는 `#[embassy_executor::main]` 매크로를 사용하여 async 환경을 설정하고, `Spawner`를 통해 태스크를 실행할 수 있어.

```rust
#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::init;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init(Default::default());

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);

    loop {
        led.set_high();
        Timer::after(Duration::from_millis(500)).await;
        led.set_low();
        Timer::after(Duration::from_millis(500)).await;
    }
}
```

이 코드는 PB5 핀에 연결된 LED를 500ms 간격으로 깜빡이게 해. `Timer::after`를 사용하여 비동기 딜레이를 구현하고 있어.

---

## 🔄 3. 버튼 입력 처리 추가

버튼 입력을 비동기적으로 처리하여 LED를 제어해보자.

```rust
use embassy_stm32::gpio::{Input, Pull};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init(Default::default());

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);
    let button = Input::new(p.PB6, Pull::Up);

    loop {
        button.wait_for_falling_edge().await;
        led.toggle();
    }
}
```

이 코드는 PB6 핀에 연결된 버튼이 눌릴 때마다 LED의 상태를 토글해. `wait_for_falling_edge`를 사용하여 버튼의 하강 에지를 비동기적으로 기다리고 있어.

---

## 🧪 4. 실습: LED 깜빡이기와 버튼 제어 통합

LED를 주기적으로 깜빡이면서, 버튼 입력에 따라 LED를 토글하는 기능을 동시에 구현해보자.

```rust
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::init;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = init(Default::default());

    let mut led = Output::new(p.PB5, Level::Low, Speed::Low);
    let button = Input::new(p.PB6, Pull::Up);

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;

        if button.is_low() {
            led.toggle();
            Timer::after(Duration::from_millis(500)).await;
        }
    }
}
```

이 코드는 LED를 500ms 간격으로 깜빡이면서, 버튼이 눌릴 때마다 추가로 LED를 토글해. `is_low`를 사용하여 버튼의 상태를 확인하고 있어.

---

## ✅ 마무리

오늘은 Embassy를 활용하여 async 환경을 설정하고, 비동기적으로 LED를 제어하고 버튼 입력을 처리하는 방법을 실습했어. `#[embassy_executor::main]` 매크로와 `Timer`, `Input` 등의 기능을 활용하여 효율적인 비동기 처리를 구현할 수 있었지.

---

## 📅 다음 시간 예고 (Day 12)

다음 시간에는 **RTIC 소개 및 비교**를 통해, Embassy와 RTIC의 차이점과 각각의 장단점을 알아볼 거야. RTIC의 `#[app]` 매크로를 사용하여 인터럽트 기반의 태스크를 작성하고, 스케줄러 개념을 이해해보자.

--- 