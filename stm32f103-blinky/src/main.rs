
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