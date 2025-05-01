# Day 7
- Timer , Interrupt 실습

```rust
static mut global_LED: Option<PC13<Output<PushPull>>> = None;
static mut global_tx : Option<Tx<USART1>> = None;
static mut global_delay : Option<SysDelay> = None;

#[entry]
fn main() -> ! {
    ~~~
    // GPIOC에서 PC13 (LED 핀) 설정
    let mut gpioc = dp.GPIOC.split();
    unsafe{ global_LED.replace(gpioc.pc13.into_push_pull_output(&mut gpioc.crh)) };

    // UART 설정
    let config_serial = Config::default().baudrate(115_200.bps()).wordlength_8bits().parity_none();
    let serial = Serial::new(dp.USART1, (pin_tx,pin_rx), &mut afio.mapr, config_serial, &clocks);
    unsafe{global_tx.replace(serial.tx)};

    //Timer 작성
    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer.listen(Event::Update);
    unsafe {NVIC::unmask(pac::Interrupt::TIM2);}//enables interrupt
    timer.start(2000.millis()).unwrap();
    
    // SYST 타이머 기반 딜레이 생성
    unsafe {global_delay.replace(Timer::syst(cp.SYST, &clocks).delay())};

    loop {
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.
        let led =  unsafe{global_LED.as_mut().unwrap()};
        let delay =  unsafe{global_delay.as_mut().unwrap()};
        
        led.set_low();    // 켜기 (PC13은 active-low)
        delay.delay_ms(500u16);

        led.set_high();   // 끄기
        delay.delay_ms(500u16);
    }
}

#[interrupt]
fn TIM2(){
    let tx =  unsafe{global_tx.as_mut().unwrap()};
    
    writeln!(tx, "hello, Rust from STM32!").unwrap();
}
```
- 이렇게 했더니 
```
error: creating a mutable reference to mutable static is discouraged
  --> src\main.rs:47:13
   |
47 |     unsafe{ global_LED.replace(gpioc.pc13.into_push_pull_output(&mut gpioc.crh)) };
   |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable reference to mutable static
   |
   = note: for more information, see <https://doc.rust-lang.org/nightly/edition-guide/rust-2024/static-mut-references.html>
   = note: mutable references to mutable statics are dangerous; it's undefined behavior if any other pointer to the static is used or if any other reference is created for the static while the mutable reference lives
   = note: `#[deny(static_mut_refs)]` on by default
```
에러가 떴다. 전역변수를 갖다가 참조해서 쓰는건 뮤텍스 위반이 될 수도 있다.(레퍼런스를 가져오는 사이에 원본이 바뀌거나 땡겨지면 안된다라)
참 다중 스레드조차 지원되지 않는 프로젝트에선 이게 왜 필요한지 모르겠다.

하여튼 mutex와 refcell과 Critical section을 이용해서 해보자.


```rust
#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use panic_halt as _;

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self,interrupt,USART1}, 
    prelude::*, 
    serial::{Config, Serial, Tx}, 
    timer::{Event, SysDelay, Timer},
};

use core::fmt::Write; //write! 매크로 사용

static global_LED   : Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static global_tx    : Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    // 디바이스와 코어 주변장치
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // 클럭 구성 (외부 8MHz 크리스탈 → 72MHz 시스템 클럭)
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

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
    let init_LED = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    //PA9 = TX, PA10 = RX
    let pin_tx = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let pin_rx  = gpioa.pa10;
    
    // UART 설정
    let config_serial = Config::default().baudrate(115_200.bps()).wordlength_8bits().parity_none();
    let serial = Serial::new(dp.USART1, (pin_tx,pin_rx), &mut afio.mapr, config_serial, &clocks);
    let init_tx = serial.tx;

    //Timer 작성
    let mut timer = dp.TIM2.counter_ms(&clocks);
    timer.listen(Event::Update);

    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);//enables interrupt
    }
    
    //timer.start(2000.millis()).unwrap();

    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();

    // critical section
    cortex_m::interrupt::free(|cs|{
        global_LED.borrow(cs).replace(Some(init_LED));
        global_tx.borrow(cs).replace(Some(init_tx));
    });
    
    cortex_m::interrupt::free(|cs| {if let Some(ref mut tx) = global_tx.borrow(cs).borrow_mut().as_mut(){
        writeln!(tx, "program starts!").unwrap();
    }});

    loop {
        
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.

        // static mut와 static의 차이때문에 unsafe가 생기고, 이걸 허용하지 않는 것이 rust이다.
        cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = global_LED.borrow(cs).borrow_mut().as_mut(){
            led.toggle();
        }
        });

        delay.delay_ms(500u16);
    }
}
#[interrupt]
fn TIM2(){
    cortex_m::interrupt::free(|cs| {if let Some(ref mut tx) = global_tx.borrow(cs).borrow_mut().as_mut(){
        writeln!(tx, "hello, Rust from STM32!").unwrap();
    }});
}
```

- timer에서 발생하는 문제임을 알았다.(comment out 결과 동작함)
- mutex부터 뚫어보자.
```rust
// critical section
cortex_m::interrupt::free(|cs|{
    global_LED.borrow(cs).replace(Some(init_LED));
    global_tx.borrow(cs).replace(Some(tx));
});
```
- (인터넷 검색)[https://doc.rust-lang.org/nomicon/panic-handler.html] 통해서 panic_handler 설치 완료.
```rust
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    cortex_m::interrupt::free(|cs| {if let Some(ref mut tx) = global_tx.borrow(cs).borrow_mut().as_mut(){
        writeln!(tx, "panic! {}",info).unwrap();
    }});


    loop {}
}
```

## 출력 매크로
```rust
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
```
- 너무 편하다. 사용할땐 `println!("$0",123);` 이런식으로 쓰면 된다.

## 타이머 프리스케일러 에러 분석
1. 10Hz 카운터 설정 상황
```rust
// main.rs:76
    let mut timer= dp.TIM2.counter::<10>(&clocks);
```
```
panic! panicked at C:\Users\Giyong\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\stm32f1xx-hal-0.10.0\src\timer.rs:684:55:<\n>
called `Result::unwrap()` on an `Err` value: TryFromIntError(())<\n>
```
2. 1ms 카운터 설정 상황
- main.rs:90 타이머 코드에서 에러.
```rust
//Timer 작성
let mut timer= dp.TIM2.counter_ms(&clocks);
```
- 에러 로그
```
panic! panicked at C:\Users\Giyong\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\stm32f1xx-hal-0.10.0\src\timer.rs:684:55:<\n>
called `Result::unwrap()` on an `Err` value: TryFromIntError(())<\n>
```
3. 에러 분석석
- `timer.rs:684:55`이게 뭘까 찾아보니
```rust
    /// Calculate prescaler depending on `Clocks` state
    pub fn configure(&mut self, clocks: &Clocks) {
        let clk = TIM::timer_clock(clocks);
        assert!(clk.raw() % FREQ == 0);
        let psc = clk.raw() / FREQ;
        self.tim.set_prescaler(u16::try_from(psc - 1)./*요 지점이 684:55*/unwrap());
    }
```
- 내부적으로 클럭이 성립하기 위한 두 가지 조건이 있다.
 1. `clk.raw() % FREQ == 0` 즉, 시스템 기본 주파수 / 타이머주파수가 나누어 떨어져야 하며
 2. `let psc = clk.raw() / FREQ; u16::try_from(psc - 1).unwrap()` 즉, 시스템 기본 주파수 / 타이머주파수의 몫이 65,536을 넘으면 안된다.
- `CounterMs<Clock>`와 `counter_ms()`를 사용하면 자동으로 Counter<T,1000> 으로 형이 지정되게 되는데, 이는 FREQ=1000을 성립하고, psc = 72000의 결과를 낳기 때문에 684:55 코드에서 unwrap() 코드에서 오버플로우 관련 내용을 처리 안 했다고 해당 에러를 표시한 것이다.

## 타이머 카운터터 에러 분석
- timer.start().unwrap()에서 에러가 났다. timer.start(int)는 Result<T,E>를 반환하는데, 여기서 에러가 발생하면 끝난다. 그래서 해결방법을 검색했다.
(방법)[https://stackoverflow.com/questions/75709115/rust-parsing-error-handling-issue-when-running-in-main]
```rust
    let res = timer.start(500.millis());
    if res.is_err() {
        cortex_m::interrupt::free(|cs| {if let Some(ref mut tx) = global_tx.borrow(cs).borrow_mut().as_mut(){
            writeln!(tx, "Error : {0:?}",res.err()).unwrap();
        }});
    }
```
근데 `Error : Some(WrongAutoReload)<\n>` 이렇게 잡아버린다. 무슨 뜻이지

- 실험 1. counter를 65536 넘어가게 만들기 : 10kHz 타이머를 10s마다 호출하게 만들면 카운터가 100,000이 되어 16비트를 넘는지 검사할 수 있다.
```rust
    println!("4");

    let res = timer.start(10000.millis());
    if let Err(e) = res {
        println!("Error : {0:?}",e);
    }
    
```
- 결과
```
3<\n>
4<\n>
Error : WrongAutoReload<\n>
5<\n>
~~~
PROGRAM STARTS!<\n>
9<\n>
```
- 역시 위 부분에서 `timer.start(t)`를 호출한 결과에서 'WrongAutoReload' 에러를 잡아냈다.

- datasheet 확인해보니, General-Purpose timer의 pres, auto-reload-counter는 **16bit**이다.

## 범용 타이머(general-purpose Timer)의 시간 설정 제약 정리리
- 프리스케일러 : `clk.raw()/FREQ` 즉 프리스케일러가 나누어떨어지면서, 그 몫이 1~65536의 범위가 되어야 한다.
- 카운터 : `millis.tick` 즉 인터럽트 주파수/타이머 주파수의 몫이 1~65536 범위가 되어야 한다.

- 이론상 최대 주기
- pres = 65536, counter = 65536
- 타이머 주기 : 1098.6328125, 인터럽트 주기 : 59.6523235...s

## Timer Interrupt Vector 필수 세팅
```rust
fn TIM2(){

    cortex_m::interrupt::free(|cs| {if let Some(ref mut timer) = global_tim2.borrow(cs).borrow_mut().as_mut(){
        //timer.tim.sr.modify(|_, w|w.uif().clear_bit());
        let res = timer.wait();
        if let Err(e) = res {
            println!("Error : {0:?}",e);
        }
    }});
}
```
- 본 코드는 공용자원에 있는 `global_tim2`를 가져왔으며, 형식은 `Counter<T,10000>`이다.
- wait 함수를 호출하여 인터럽트 카운터 플래그를 초기화한다.(그래야지 다시 셀 수가 있지 안그러면 그다음 카운터 Tick마다 counter=1으로 가정하고 의도한것보다 빠르게 호출된다.)
- 레지스터를 직접 건드리는 방법이 있는데, `timer.tim.sr.modify(|_, w|w.uif().clear_bit());`이다.

## Day 7 최종 코드
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

// 전역 자원: Mutex + RefCell로 안전하게 보호
static global_LED: Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static global_tx: Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));
static global_tim2: Mutex<RefCell<Option<Counter<TIM2, 10000>>>> = Mutex::new(RefCell::new(None));

// 📦 출력 매크로 정의 (인터럽트 안전 구역에서 사용)
macro_rules! println {
    ($($arg:tt)*) => {
        cortex_m::interrupt::free(|cs| {
            if let Some(tx) = global_tx.borrow(cs).borrow_mut().as_mut() {
                writeln!(tx, $($arg)*).ok();
            }
        })
    };
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let cp = cortex_m::Peripherals::take().unwrap();

    // 시스템 클럭: 8MHz 외부 크리스탈 → 72MHz SYSCLK
    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .pclk1(36.MHz())
        .freeze(&mut flash.acr);

    // GPIO: PC13 (LED 출력용)
    let mut gpioc = dp.GPIOC.split();
    let led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // UART: PA9 (TX), PA10 (RX)
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let tx_pin = gpioa.pa9.into_alternate_push_pull(&mut gpioa.crh);
    let rx_pin = gpioa.pa10;

    let serial = Serial::new(
        dp.USART1,
        (tx_pin, rx_pin),
        &mut afio.mapr,
        Config::default()
            .baudrate(115_200.bps())
            .wordlength_8bits()
            .parity_none(),
        &clocks
    );
    let (tx, _rx) = serial.split();

    // 💾 공유 자원 등록
    cortex_m::interrupt::free(|cs| {
        global_LED.borrow(cs).replace(Some(led));
        global_tx.borrow(cs).replace(Some(tx));
    });

    println!("2");

    // TIM2 타이머 설정: 10kHz 기준, 1.5초마다 인터럽트
    let mut timer = dp.TIM2.counter::<10_000>(&clocks);
    println!("3");
    timer.listen(Event::Update);
    println!("4");

    let res = timer.start(1_500_000.micros());
    if let Err(e) = res {
        println!("Error : {0:?}", e);
    }
    println!("5");

    cortex_m::interrupt::free(|cs| {
        global_tim2.borrow(cs).replace(Some(timer));
    });
    println!("6");

    // NVIC에서 TIM2 인터럽트 활성화
    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);
    }
    println!("7");

    // 💡 메인 루프용 SYST 타이머 딜레이 설정
    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();
    println!("8");

    println!("PROGRAM STARTS!");

    loop {
        println!("9");

        // LED 토글 (인터럽트 안전하게 보호)
        cortex_m::interrupt::free(|cs| {
            if let Some(led) = global_LED.borrow(cs).borrow_mut().as_mut() {
                led.toggle();
            }
        });

        delay.delay_ms(500_u16);
    }
}

#[interrupt]
fn TIM2() {
    // 인터럽트에서 UART 출력
    println!("hello, Rust from STM32!");

    // 플래그 clear (또는 wait() 호출)
    cortex_m::interrupt::free(|cs| {
        if let Some(timer) = global_tim2.borrow(cs).borrow_mut().as_mut() {
            let res = timer.wait();
            if let Err(e) = res {
                println!("Error : {0:?}", e);
            }
        }
    });
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("panic! {}", info);
    loop {}
}

```

- 출력
```
2
3
4
5
6
7
8
PROGRAM STARTS!
9
9
9
hello, Rust from STM32!
9
9
9
hello, Rust from STM32!
```
- main LED 점멸 및 "9" 출력은 500ms마다, Interrupt는 1500ms마다 이루어지므로, 9 출력 3번에 hello 문구가 나타나는것을 알 수 있다.


## (footer note) 빌드 및 플래시 명령어

작성한 Rust 코드를 바이너리로 컴파일하고 보드에 업로드하는 명령어는 다음과 같습니다:

```bash
cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/stm32f103-blinky firmware.bin
arm-none-eabi-size target/thumbv7m-none-eabi/release/stm32f103-blinky
```

## Set of Using Timer
```rust

#[interrupt]




    //Timer 작성
    let mut timer= dp.TIM2.counter::<10_000>(&clocks);

    timer.listen(Event::Update);


    let res = timer.start(1_500_000.micros());
    if let Err(e) = res {
        println!("Error : {0:?}",e);
    }

    // critical section
    cortex_m::interrupt::free(|cs|{
        global_tim2.borrow(cs).replace(Some(timer));
    });


    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);//enables interrupt
    }
    


fn TIM2(){
    println!("hello, Rust from STM32!");
    /*
    cortex_m::interrupt::free(|cs| {if let Some(ref mut timer) = global_tim2.borrow(cs).borrow_mut().as_mut(){
        //timer.tim.sr.modify(|_, w|w.uif().clear_bit());
        let _ = timer.wait();
    }});
    */
    
    // TIM2 레지스터에 직접 접근하여 플래그 초기화
    // SR 레지스터의 UIF(Update Interrupt Flag) 비트를 0으로 설정하여 초기화
    /*unsafe {
        (*stm32f1xx_hal::device::TIM2::ptr()).sr.modify(|_, w| w.uif().clear());
    }*/
    cortex_m::interrupt::free(|cs| {if let Some(ref mut timer) = global_tim2.borrow(cs).borrow_mut().as_mut(){
        //timer.tim.sr.modify(|_, w|w.uif().clear_bit());
        let res = timer.wait();
        if let Err(e) = res {
            println!("Error : {0:?}",e);
        }
    }});

}

```