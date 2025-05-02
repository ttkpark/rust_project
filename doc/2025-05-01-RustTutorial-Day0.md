아주 중요하고 정확한 지적이야.  
# 📘 STM32F1 기반 Rust 입문 교육과정 (14일 x 1시간)

- **대상**: 기존 C/STM32Cube 기반의 STM32 개발자
- **목표**: Rust 임베디드 개발에 대한 감각을 빠르게 익히고, 실무 C 프로젝트를 Rust로 옮기는 기반 확보
- **보드**: STM32F103C8T6 (Blue Pill, ST-Link 또는 USB-Serial 사용)

---

| Day | 주제 | 목표 |
|-----|------|------|
| 1 | Rust와 임베디드 개요 | Rust 철학과 C 비교, embedded 장점, memory safety 이해 |
| 2 | 개발 환경 구축 | Rust toolchain 설치, target 추가, `cargo` 사용법, `probe-rs` 설정 |
| 3 | STM32F1 구조 & CubeMX 활용 | STM32F1 구조 이해, 핀/클럭 설정, CubeMX로 `.ioc` 참고 |
| 4 | 첫 Rust 프로젝트 | GPIO 출력 (LED Blink), `delay` 사용 |
| 5 | 메모리/링커스크립트 구조 | `no_std`, `no_main`, `memory.x`, 빌드 흐름 파악 |
| 6 | HAL 구조 분석 | `stm32f1xx-hal` 구조와 주변장치 초기화 방식 이해 |
| 7 | UART 송수신 실습 | TX/RX 설정, `writeln!()`, 인터럽트 기반 수신 |
| 8 | 타이머 + 인터럽트 실습 | `Timer`, NVIC, 주기적 인터럽트 핸들러 |
| 9 | GPIO 입력 + 디바운싱 | 버튼 입력 (EXTI), 소프트웨어 디바운싱 적용 |
| 10 | RTOS 개요 & embassy 소개 | FreeRTOS 구조, embassy 개념, async executor 도입 |
| 11 | embassy Task 구조 실습 | `#[embassy_executor::main]`, async task, delay, input 처리 |
| 12 | RTIC 소개 및 비교 | `#[app]`, 리소스 관리, RTIC vs embassy 비교 |
| 13 | 기존 C 프로젝트 포팅 | GPIO + UART + Timer 기반 프로젝트 Rust로 재현 |
| 14 | 마무리 + 실무 적용 전략 | crate 선택법, 예외 처리, 구조화 전략, 빌드 시스템 정리 |

---

## 📁 구성 방식

- **15분**: 개념 설명 (Rust/임베디드/HAL 구조 등)
- **30분**: 실습 코드 작성 및 업로드
- **15분**: 질문, 리뷰, 실무 적용 방향 안내

---

## 📦 사용 툴 (STM32F103 기준)

| 항목 | 사용 내용 |
|------|-----------|
| Rust Toolchain | `rustup`, `cargo`, `thumbv7m-none-eabi` |
| HAL Crate | `stm32f1xx-hal = "0.10.0"` |
| Flash 도구 | `probe-rs`, `cargo-flash` 또는 `STM32CubeProgrammer` |
| 디버깅 | `cargo-embed` + GDB |
| IDE | VSCode + `rust-analyzer` |
| STM32CubeMX | 핀맵/클럭 참고용 GUI (자동 코드 생성은 안 씀) |

---

## 🧠 RTOS & Task 대체 전략 (STM32Cube → Rust)

| 기능                       | STM32Cube(C) | Rust (대체) | 지원 |
|----------------------------|--------------|-------------|------|
| FreeRTOS Task 관리         | GUI + 설정   | `embassy` async / `RTIC` | ✅ |
| SysTick 타이머             | 자동 설정    | `Timer::syst()` / `SysDelay` | ✅ |
| 인터럽트 우선순위         | NVIC config  | `#[interrupt]` + NVIC 직접 설정 | ✅ |
| Task 간 통신 (Queue 등)   | Queue        | `channel`, `static mut`, `Mutex<RefCell>` | ✅ |
| 프로젝트 구조화           | Makefile     | `cargo`, `workspace`, `bin` | ✅ |

---

## ✨ Rust에서 RTOS를 쓰지 않고도 할 수 있는 일

| 기능 | 방법 |
|------|------|
| 다중 태스크 | `embassy`의 `async fn`, `select!`, `spawn` |
| 실시간 인터럽트 + 스케줄링 | `RTIC` 프레임워크 (`#[app]`) |
| 공유 리소스 제어 | `Mutex<RefCell<T>>`, `static mut`, `CriticalSection` |
| 디바운싱, 딜레이 | `Timer`, `SysDelay`, `cortex_m::asm::delay` |

---

## 📌 Rust → C 프로젝트 포팅 흐름 예시 (Day 13)

| 기능 | C 코드 | Rust 코드 |
|------|--------|-----------|
| LED Blink | HAL_GPIO_WritePin | `led.set_high()` |
| UART | HAL_UART_Transmit | `writeln!(tx, "...")` |
| Timer | HAL_TIM_Base_Start_IT | `Timer::counter().listen(...)` |
| Interrupt | `void HAL_GPIO_EXTI_Callback()` | `#[interrupt] fn EXTIx()` |

---

## ✅ 이후 흐름 확장 (진도 끝난 후)

- `embassy` + `embassy-stm32` 로 대규모 async 구조 도입
- `defmt` + `probe-rs` 활용한 로깅 디버깅 환경 구축
- `RTIC`으로 실시간 task 관리 구조 정리

---

이제 이 강의계획서를 기반으로, Day 10부터는 계획대로:

- **RTOS 개요**
- **FreeRTOS 구조 vs embassy 구조 비교**
- **executor / async fn / spawn 개념 소개**

로 진행할 수 있어. 필요하면 해당 내용을 실제 STM32F103에서 동작하도록 embassy 설정도 도와줄게.

이제 Rust를 진짜 실무 프로젝트에 적용할 기반이 충분히 준비된 거야.  
계속 이렇게만 하면, 네가 갖고 있는 C 기반 STM32 프로젝트도 자연스럽게 Rust로 포팅할 수 있게 될 거야! 🚀
