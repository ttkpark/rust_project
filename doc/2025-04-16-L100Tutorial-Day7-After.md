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

- timer 조작 결과
```rust
// main.rs:76
    let mut timer= dp.TIM2.counter::<10>(&clocks);
```
```
panic! panicked at C:\Users\Giyong\.cargo\registry\src\index.crates.io-1949cf8c6b5b557f\stm32f1xx-hal-0.10.0\src\timer.rs:684:55:<\n>
called `Result::unwrap()` on an `Err` value: TryFromIntError(())<\n>
```
 잘 보면 어느 스케일러 조합으로도 10Hz 타이머는 못 만들어내니까 나오는 에러다. 너무 추상화가 되어 있어서 오히려 불편하다.

## 해결


```rust
#[interrupt]
fn TIM2() {
    interrupt::free(|cs| {
        ......

        // 인터럽트 플래그 clear
        let tim2 = unsafe { &*TIM2::ptr() };
        tim2.sr.modify(|_, w| w.uif().clear_bit());
    });
}
```
- 인터럽트 플래그를 저렇게 두니까 이상했다.

 하지만, 주어진 코드도 동작하지 않았고, 컴파일 문제를 해결해 나갔다.
 1. TIM2가 존재하지 않았다 : `pac::Peripherals::take().unwrap()`를 pac로 저장하고 각종 컴포넌트(GPIO, UART 등)에 접근하는 핸들(GPIOA, UART1 등)을 보고 저것을 모방해서 가져와야되나 싶었다.
```rust
let tim2 = pac::Peripherals::take().unwrap().TIM2;
tim2.sr.modify(|_, w| w.uif().clear_bit());
```
- 결론 성공! 이젠 500ms마다 깜빡인다.


## 6. 빌드 및 플래시 명령어

작성한 Rust 코드를 바이너리로 컴파일하고 보드에 업로드하는 명령어는 다음과 같습니다:

```bash
cargo build --release
arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/stm32f103-blinky firmware.bin
arm-none-eabi-size target/thumbv7m-none-eabi/release/stm32f103-blinky
```