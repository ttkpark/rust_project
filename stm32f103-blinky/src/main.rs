#![no_std]
#![no_main]

use cortex_m::peripheral::NVIC;
use core::cell::RefCell;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
//use panic_halt as _;
use core::panic::PanicInfo;

use stm32f1xx_hal::{
    gpio::{Output, PushPull, *},
    pac::{self, interrupt, TIM2, USART1}, 
    prelude::*, 
    serial::{Config, Serial, Tx}, 
    timer::{Counter, Event, SysDelay, Timer}, //Tim2NoRemap 뭔뜻이지
};


use core::fmt::Write; //write! 매크로 사용
static global_LED   : Mutex<RefCell<Option<PC13<Output<PushPull>>>>> = Mutex::new(RefCell::new(None));
static global_tx    : Mutex<RefCell<Option<Tx<USART1>>>> = Mutex::new(RefCell::new(None));

// 글로벌 타이머 인스턴스 변수 선언
static global_tim2: Mutex<RefCell<Option<Counter<TIM2,10000>>>> = Mutex::new(RefCell::new(None));

// 출력용 매크로 정의
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
    let mut init_LED: PC13<Output<PushPull>> = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    // GPIO 설정
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();

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

    // critical section
    cortex_m::interrupt::free(|cs|{
        global_LED.borrow(cs).replace(Some(init_LED));
        global_tx.borrow(cs).replace(Some(tx));
        //*GLOBAL_TX.borrow(cs).borrow_mut() = Some(tx); 
    });
    
    println!("2");

    //Timer 작성
    let mut timer= dp.TIM2.counter::<10_000>(&clocks);

    println!("3");

    timer.listen(Event::Update);

    println!("4");

    let res = timer.start(1_500_000.micros());
    if let Err(e) = res {
        println!("Error : {0:?}",e);
    }
    
    println!("5");

    // critical section
    cortex_m::interrupt::free(|cs|{
        global_tim2.borrow(cs).replace(Some(timer));
    });
    
    println!("6");

    unsafe {
        NVIC::unmask(pac::Interrupt::TIM2);//enables interrupt
    }
    
    println!("7");

    let mut delay: SysDelay = Timer::syst(cp.SYST, &clocks).delay();
    
    println!("8");

    
    println!("PROGRAM STARTS!");

    loop {
        
        println!("9");
        // rust는 소유권이란게 있어서, 1대1 대응하면 이전 변수는 더이상 사용할 수 없다. (A=B 했을 때 clone이 아니라 소유권 이전이 된다.)
        // 전역변수 LED를 꺼낼 땐 소유권 이전할 수도 없고 필요도 없어서 mutable 참조만 꺼낼 수가 있는데 그게 as_mut()이다.

        // static mut와 static의 차이때문에 unsafe가 생기고, 이걸 허용하지 않는 것이 rust이다.
        cortex_m::interrupt::free(|cs| {
        if let Some(ref mut led) = global_LED.borrow(cs).borrow_mut().as_mut(){
            led.toggle();
        }
        });
        //init_LED.toggle();

        delay.delay_ms(500u16);
    }
}
#[interrupt]
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
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    
    // logs "panicked at '$reason', src/main.rs:27:4" to the host stderr
    println!("panic! {}",info);

    loop {}
}