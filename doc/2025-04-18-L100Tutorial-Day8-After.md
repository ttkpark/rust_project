# Interrupt of 외부 입력

## 구현 사항

```rust

use cortex_m::interrupt::Mutex;
use cortex_m_rt::entry;
use core::cell::RefCell;
use core::panic::PanicInfo;

use stm32f1xx_hal::{
    gpio::{Edge, ExtiPin, Input, PullUp, gpiob::PB0, gpioc::PC13, Output, PushPull},
    pac::{self, interrupt, EXTI, USART1},
    prelude::*,
    serial::{Serial, Config, Tx},
    timer::{self,SysDelay,Timer}
};
static GLOBAL_BTN: Mutex<RefCell<Option<PB0<Input<PullUp>>>>> = Mutex::new(RefCell::new(None));


    let mut gpiob = dp.GPIOB.split();
    let mut button = gpiob.pb0.into_pull_up_input(&mut gpiob.crl);

    // NVIC 인터럽트 언마스크
    unsafe {
        cortex_m::peripheral::NVIC::unmask(pac::Interrupt::EXTI0);
    }
    
    println!("2");

    // 외부 인터럽트 설정
    let mut exti = dp.EXTI;
    button.make_interrupt_source(&mut afio);
    button.trigger_on_edge(&mut exti, Edge::Falling); // 버튼 눌림에 반응
    button.enable_interrupt(&mut exti);

    // critical section
    cortex_m::interrupt::free(|cs|{
        GLOBAL_BTN.borrow(cs).replace(Some(button));
    });


#[interrupt]
fn EXTI0() {
    println!("Button pressed!");

    cortex_m::interrupt::free(|cs| {
        if let Some(led) = GLOBAL_LED.borrow(cs).borrow_mut().as_mut() {
            led.toggle();
        }

        if let Some(btn) = GLOBAL_BTN.borrow(cs).borrow_mut().as_mut() {
            btn.clear_interrupt_pending_bit(); // 인터럽트 플래그 clear
        }
    });
}

```