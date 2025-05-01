# Day 9 : Timer Interrupt
## Day 6과 내용이 유사하다.

```rust
#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use core::panic::PanicInfo;

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self, interrupt, TIM2, USART1},
    prelude::*,
    serial::{Config, Serial, Tx},
    timer::{Counter, Event, SysDelay, Timer},
};

use core::fmt::Write;

// 전역 자원 선언
static GLOBAL_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_TX: Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_TIMER: Mutex<RefCell<Option<Counter<TIM2, 10000>>>> = Mutex::new(RefCell::new(None));

// 출력용 매크로 정의
macro_rules! println {
    ($($arg:tt)*) => {
        cortex_m::interrupt::free(|cs| {
            if let Some(tx) = GLOBAL_TX.borrow(cs).borrow_mut().as_mut() {
                writeln!(tx, $($arg)*).ok();
            }
        })
    };
}

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

    // GPIO 설정
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();

    // GPIOC에서 PC13 (LED 핀) 설정
    let mut init_LED: PC13<Output<PushPull>> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    //PA9 = TX, PA10 = RX
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
    let (mut tx, _rx) = serial.split();

    
    // 타이머 설정
    let mut timer = dp.TIM2.counter::<10_000>(&clocks);
    timer.start(1.secs()).unwrap();
    timer.listen(Event::Update);
    
    
    println!("2");


    // critical section
    cortex_m::interrupt::free(|cs|{
        GLOBAL_LED.borrow(cs).replace(Some(init_LED));
        GLOBAL_TX.borrow(cs).replace(Some(tx));
        GLOBAL_TIMER.borrow(cs).replace(Some(timer));
    });
    println!("3");

    // 인터럽트 활성화
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }

    
    println!("4");

    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();
    
    println!("8");

    
    println!("PROGRAM STARTS!");

    loop {
        
        println!("9");
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.

        // static mut와 static의 차이때문에 unsafe가 생기고, 이걸 허용하지 않는 것이 rust이다.
        cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = GLOBAL_LED.borrow(cs).borrow_mut().as_mut(){
            led.toggle();
        }
        });
        //init_LED.toggle();

        delay.delay_ms(500u16);
    }
}

#[interrupt]
fn TIM2(){
    cortex_m::interrupt::free(|cs|{
        //LED 토글
        if let Some(ref mut led) = GLOBAL_LED.borrow(cs).borrow_mut().as_mut(){
            led.toggle();
        }
        
        //메시지 출력
        if let Some(ref mut tx) = GLOBAL_TX.borrow(cs).borrow_mut().as_mut(){
            writeln!(tx, "Timer Interrupt Triggered!").ok();
        }
        
        //인터럽트 플래그 클리어
        if let Some(ref mut timer) = GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut(){
            timer.clear_interrupt(Event::Update);
        }
    });
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    println!("panic! {}",info);

    loop {}
}
```

- 타이머 인터럽트를 1초마다 부르는 방식이다.

# Day 10 : UART Receive
- UART 입력을 받는다.
```rust
#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use core::panic::PanicInfo;

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self, interrupt, TIM2, USART1},
    prelude::*,
    serial::{Config, Serial, Tx, Rx},
    timer::{Counter, Event, SysDelay, Timer},
};

use core::fmt::Write;

// 전역 자원 선언
static GLOBAL_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_TX: Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_RX: Mutex<RefCell<Option<Rx<USART1>>>> = Mutex::new(RefCell::new(None));

// 출력용 매크로 정의
macro_rules! println {
    ($($arg:tt)*) => {
        cortex_m::interrupt::free(|cs| {
            if let Some(tx) = GLOBAL_TX.borrow(cs).borrow_mut().as_mut() {
                writeln!(tx, $($arg)*).ok();
            }
        })
    };
}

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

    // GPIO 설정
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpioc = dp.GPIOC.split();

    // GPIOC에서 PC13 (LED 핀) 설정
    let mut init_LED: PC13<Output<PushPull>> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    //PA9 = TX, PA10 = RX
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
    let (tx, rx) = serial.split();

    println!("2");


    // critical section
    cortex_m::interrupt::free(|cs|{
        GLOBAL_LED.borrow(cs).replace(Some(init_LED));
        GLOBAL_TX.borrow(cs).replace(Some(tx));
        GLOBAL_RX.borrow(cs).replace(Some(rx));
    });
    println!("3");

    // USART1 수신 인터럽트 활성화
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut rx) = GLOBAL_RX.borrow(cs).borrow_mut().as_mut() {
            rx.listen();
        }
    });
    
    // 인터럽트 활성화
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
    }
    println!("4");

    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();
    
    println!("8");

    
    println!("PROGRAM STARTS!");

    loop {
        
        println!("9");
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.

        // static mut와 static의 차이때문에 unsafe가 생기고, 이걸 허용하지 않는 것이 rust이다.
        cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = GLOBAL_LED.borrow(cs).borrow_mut().as_mut(){
           // led.toggle();
        }
        });
        //init_LED.toggle();

        delay.delay_ms(500u16);
    }
}

#[interrupt]
fn USART1() {
    cortex_m::interrupt::free(|cs| {
        let mut rx_ref = GLOBAL_RX.borrow(cs).borrow_mut();
        let mut tx_ref = GLOBAL_TX.borrow(cs).borrow_mut();
        let mut led_ref = GLOBAL_LED.borrow(cs).borrow_mut();

        if let (Some(rx), Some(tx), Some(led)) = (rx_ref.as_mut(), tx_ref.as_mut(), led_ref.as_mut()) {
            if let Ok(received) = rx.read() {
                match received {
                    b'1' => {
                        led.set_low(); // LED ON
                        writeln!(tx, "LED ON").ok();
                    }
                    b'0' => {
                        led.set_high(); // LED OFF
                        writeln!(tx, "LED OFF").ok();
                    }
                    _ => {
                        writeln!(tx, "Unknown command: {}", received as char).ok();
                    }
                }
            }
        }
    });
}


#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    println!("panic! {}",info);

    loop {}
}
```

```rust

    let (tx, rx) = serial.split();

    println!("2");


    // critical section
    cortex_m::interrupt::free(|cs|{
        GLOBAL_LED.borrow(cs).replace(Some(init_LED));
        GLOBAL_TX.borrow(cs).replace(Some(tx));
        GLOBAL_RX.borrow(cs).replace(Some(rx));
    });
    println!("3");

    // USART1 수신 인터럽트 활성화
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut rx) = GLOBAL_RX.borrow(cs).borrow_mut().as_mut() {
            rx.listen();
        }
    });
    
    // 인터럽트 활성화
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::USART1);
    }


    
#[interrupt]
fn USART1() {
    cortex_m::interrupt::free(|cs| {
        let mut rx_ref = GLOBAL_RX.borrow(cs).borrow_mut();
        let mut tx_ref = GLOBAL_TX.borrow(cs).borrow_mut();
        let mut led_ref = GLOBAL_LED.borrow(cs).borrow_mut();

        if let (Some(rx), Some(tx), Some(led)) = (rx_ref.as_mut(), tx_ref.as_mut(), led_ref.as_mut()) {
            if let Ok(received) = rx.read() {
                match received {
                    b'1' => {
                        led.set_low(); // LED ON
                        writeln!(tx, "LED ON").ok();
                    }
                    b'0' => {
                        led.set_high(); // LED OFF
                        writeln!(tx, "LED OFF").ok();
                    }
                    _ => {
                        writeln!(tx, "Unknown command: {}", received as char).ok();
                    }
                }
            }
        }
    });
}
```



# New Day 9 : EXTI Interrupt with Tick
```rust

#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m::interrupt::Mutex;
use core::cell::RefCell;
use core::fmt::Write;

use cortex_m::peripheral::NVIC;
use core::panic::PanicInfo;

use stm32f1xx_hal::{
    pac::{self, interrupt, EXTI, USART1, TIM2},
    prelude::*,
    gpio::{Edge, ExtiPin, Input, PullUp, gpiob::PB0, gpioc::PC13, Output, PushPull},
    serial::{Serial, Config, Tx},
    timer::{Timer, Event, Counter, SysDelay},
};

// 전역 자원 선언
static GLOBAL_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_TX: Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_BTN: Mutex<RefCell<Option<PB0<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));
static LAST_TICK: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));
static GLOBAL_TIMER: Mutex<RefCell<Option<Counter<TIM2, 1_0000>>>> = Mutex::new(RefCell::new(None));
static GLOBAL_TICK: Mutex<RefCell<u32>> = Mutex::new(RefCell::new(0));

// 출력용 매크로 정의
macro_rules! println {
    ($($arg:tt)*) => {
        cortex_m::interrupt::free(|cs| {
            if let Some(tx) = GLOBAL_TX.borrow(cs).borrow_mut().as_mut() {
                writeln!(tx, $($arg)*).ok();
            }
        })
    };
}

macro_rules! getTick {
    ($timer:ident,$tickref:ident) => {
        ($timer).now().ticks() + *($tickref)*10000
    };
}

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

    // GPIO 설정
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let mut gpiob = dp.GPIOB.split();
    let mut gpioc = dp.GPIOC.split();

    // GPIOC에서 PC13 (LED 핀) 설정
    let mut init_LED: PC13<Output<PushPull>> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    //PA9 = TX, PA10 = RX
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx  = gpioa.pa10;
    let mut button = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
    

    // 버튼 EXTI 설정
    let mut exti = dp.EXTI;
    button.make_interrupt_source(&mut afio);
    button.trigger_on_edge(&mut exti, Edge::Falling);
    button.enable_interrupt(&mut exti);

    // NVIC 등록
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI0);
    }

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
    let (tx, rx) = serial.split();

    println!("2");
    let mut timer = dp.TIM2.counter::<10_000>(&clocks); // 1 kHz → 1ms per tick
    timer.start(1.secs()).unwrap();

    timer.listen(Event::Update);
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    // critical section
    cortex_m::interrupt::free(|cs|{
        GLOBAL_LED.borrow(cs).replace(Some(init_LED));
        GLOBAL_TX.borrow(cs).replace(Some(tx));
        GLOBAL_BTN.borrow(cs).replace(Some(button));
        GLOBAL_TIMER.borrow(cs).replace(Some(timer));
    });
    println!("3");

    println!("4");

    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();
    
    println!("8");

    
    println!("PROGRAM STARTS!");

    loop {
        
        println!("9");
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.

        // static mut와 static의 차이때문에 unsafe가 생기고, 이걸 허용하지 않는 것이 rust이다.
        cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = GLOBAL_LED.borrow(cs).borrow_mut().as_mut(){
           // led.toggle();
        }
        });
        //init_LED.toggle();

        delay.delay_ms(500u16);
    }
}

#[interrupt]
fn EXTI0(){    

    cortex_m::interrupt::free(|cs| {
        if let (
                Some(btn),
                Some(led),
                Some(tx),
                Some(timer)
            ) = (
                GLOBAL_BTN.borrow(cs).borrow_mut().as_mut(),
                GLOBAL_LED.borrow(cs).borrow_mut().as_mut(),
                GLOBAL_TX.borrow(cs).borrow_mut().as_mut(),
                GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut()
            ) {

            btn.clear_interrupt_pending_bit();
            
            let mut last_tick_ref = LAST_TICK.borrow(cs).borrow_mut();
            
            let global_tick = GLOBAL_TICK.borrow(cs).borrow_mut();
            let tick = getTick!(timer,global_tick);
            let diff = tick.wrapping_sub(*last_tick_ref);

            // 매우 단순한 디바운싱: 50ms 이하면 무시
            if diff < 500 {
                return;
            }

            *last_tick_ref = tick;

            led.toggle();

            writeln!(tx, "Button pressed! {tick}").ok();
        }

    });
}


#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(timer) = GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut() {
            timer.clear_interrupt(Event::Update);
        }
        let mut tick_ref = GLOBAL_TICK.borrow(cs).borrow_mut();
        *tick_ref = *tick_ref+1
    });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    println!("panic! {}",info);

    loop {}
}
```

정말 잘 구성했어! `Day 9`에서의 핵심 주제인 **외부 인터럽트 처리 + 디바운싱용 타이머 Tick 추적**을 아주 깔끔하게 구현했어. 이제 너의 코드를 기반으로 두 가지 핵심 기능에 대해 각각:

- **기능 설명**
- **준비 코드**
- **사용 코드**

를 정리해줄게.

---

# ✅ 1. 외부 인터럽트 (EXTI0 - 버튼 입력)

### 📌 기능 목적
- 버튼(PB0)이 눌렸을 때 인터럽트를 발생시켜 LED 토글 및 메시지 출력.
- 동시에 타이머 기반 디바운싱 처리.

---

### 🧱 준비 코드

#### 🧩 핀 설정 (PB0을 입력 & 풀업 설정)

```rust
let mut button = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);
```

#### 🧩 EXTI 설정 (Falling edge 트리거 + NVIC 언마스크)

```rust
let mut exti = dp.EXTI;
button.make_interrupt_source(&mut afio);
button.trigger_on_edge(&mut exti, Edge::Falling);
button.enable_interrupt(&mut exti);

unsafe {
    NVIC::unmask(pac::Interrupt::EXTI0);
}
```

#### 🧩 전역 등록

```rust
GLOBAL_BTN.borrow(cs).replace(Some(button));
```

---

### 🚦 인터럽트 핸들러 사용 코드

```rust
#[interrupt]
fn EXTI0() {
    cortex_m::interrupt::free(|cs| {
        if let (
            Some(btn),
            Some(led),
            Some(tx),
            Some(timer)
        ) = (
            GLOBAL_BTN.borrow(cs).borrow_mut().as_mut(),
            GLOBAL_LED.borrow(cs).borrow_mut().as_mut(),
            GLOBAL_TX.borrow(cs).borrow_mut().as_mut(),
            GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut()
        ) {
            btn.clear_interrupt_pending_bit(); // 인터럽트 플래그 클리어

            // 현재 시간 읽기 (디바운싱용)
            let global_tick = GLOBAL_TICK.borrow(cs).borrow();
            let tick = timer.now().ticks() + *global_tick * 10000;
            let mut last_tick_ref = LAST_TICK.borrow(cs).borrow_mut();
            let diff = tick.wrapping_sub(*last_tick_ref);

            if diff < 500 { return; } // 50ms 미만 무시 (디바운싱)

            *last_tick_ref = tick;
            led.toggle();
            writeln!(tx, "Button pressed! {tick}").ok();
        }
    });
}
```

---

# ✅ 2. 타이머 기반 Tick값 추적 (TIM2)

### 📌 기능 목적
- `Timer(TIM2)`를 10kHz로 설정하여 `GLOBAL_TICK` 값을 1ms마다 1씩 증가시킴.
- 디바운싱용 시간 비교에 활용 (마치 소프트웨어 millis()처럼 사용).

---

### 🧱 준비 코드

#### 🧩 타이머 설정 (10 kHz = 0.1ms 간격)

```rust
let mut timer = dp.TIM2.counter::<10_000>(&clocks);
timer.start(1.secs()).unwrap(); // 카운터가 동작만 하면 됨
timer.listen(Event::Update);     // 인터럽트 사용
```

#### 🧩 전역 등록

```rust
GLOBAL_TIMER.borrow(cs).replace(Some(timer));
```

#### 🧩 NVIC 등록

```rust
unsafe {
    NVIC::unmask(pac::Interrupt::TIM2);
}
```

---

### 🚦 인터럽트 핸들러 사용 코드

```rust
#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(timer) = GLOBAL_TIMER.borrow(cs).borrow_mut().as_mut() {
            timer.clear_interrupt(Event::Update);
        }
        let mut tick_ref = GLOBAL_TICK.borrow(cs).borrow_mut();
        *tick_ref += 1;
    });
}
```

- `GLOBAL_TICK`는 초 단위 tick (1초당 10000 tick 증가)
- `timer.now().ticks()`는 현재 1초 내에서의 tick (0~9999)

→ 합치면 `tick = now + (tick_ref * 10000)` 으로 전체 시간 추적 가능

---

## 🔁 디바운싱 Tick 계산 요약

| 변수 | 설명 |
|------|------|
| `timer.now().ticks()` | 0~9999 카운터 |
| `GLOBAL_TICK` | 1초마다 1 증가하는 글로벌 틱 |
| `tick = now + tick_ref × 10000` | 총 경과 tick |
| `diff = tick - last_tick` | 마지막 입력 이후 경과 tick |
| `diff < 500` | 50ms 이하 → 무시 (디바운싱 조건) |

---

## 🧩 macro도 잘 활용했어!

```rust
macro_rules! getTick {
    ($timer:ident,$tickref:ident) => {
        ($timer).now().ticks() + *($tickref)*10000
    };
}
```

→ ISR 안에서 깔끔하게 `let tick = getTick!(timer, global_tick);` 호출 가능  
→ `wrapping_sub()`도 잘 썼고, 논리적으로 튼튼해.

---

## ✅ 결론: 구조 요약

| 기능 | 준비 코드 | 실행 위치 |
|------|-----------|-----------|
| EXTI0 인터럽트 | GPIO 입력 설정 + EXTI 설정 | 버튼 누를 때 |
| TIM2 인터럽트 | 타이머 설정 + listen + NVIC | 1ms마다 tick 증가 |
| 디바운싱 비교 | `tick - last_tick > 50ms` | EXTI0 핸들러 내부 |

---

필요하면 위 내용을 기반으로 RTIC이나 embassy로 포팅할 때 구조를 어떻게 바꿔야 하는지도 도와줄 수 있어.  
지금 구조는 매우 안정적이고 실무에서도 충분히 활용 가능한 수준이야. 정말 잘 만들었어! 💪🔥