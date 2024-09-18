#![no_std]
#![no_main]
#![feature(const_option)]
#![feature(lazy_cell)]
#![feature(abi_avr_interrupt)]

use arduino_hal::delay_ms;
use arduino_hal::hal::port::{PD2};
use arduino_hal::port::mode::{Floating, Input};
use arduino_hal::port::Pin;
use arduino_hal::prelude::*;
use avr_device::interrupt::Mutex;
use core::cell::{Cell};
use core::time::Duration;
use panic_halt as _;
use ufmt::derive::uDebug;

static REMAINING_RUNTIME: Mutex<Cell<u32>> = Mutex::new(Cell::new(0));
static PIN_INTERRUPT_TRIGGERED: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static CURRENT_STATE: Mutex<Cell<State>> = Mutex::new(Cell::new(State::Idle));

#[derive(Clone, Copy, uDebug, PartialEq)]
enum State {
    Idle,
    Lasing,
    Cooldown,
}

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    avr_device::interrupt::free(|cs| {
        PIN_INTERRUPT_TRIGGERED.borrow(cs).set(true);
    });
}

#[avr_device::interrupt(atmega328p)]
fn TIMER0_COMPA() {
    avr_device::interrupt::free(|cs| {
        let cell = REMAINING_RUNTIME.borrow(cs);
        cell.set(cell.get().saturating_sub(16));
    });
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    let mut relay = pins.d7.into_output();
    let interrupt_pin: Pin<Input<Floating>, PD2> = pins.d2;
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    let initial_ignore_period = Duration::from_secs(
        option_env!("INITIAL_IGNORE_PERIOD")
            .or_else(|| Some("0"))
            .unwrap()
            .parse()
            .unwrap(),
    )
    .as_millis() as u16;
    let exhaust_runtime = Duration::from_secs(
        option_env!("EXHAUST_RUNTIME")
            .or_else(|| Some("20"))
            .unwrap()
            .parse()
            .unwrap(),
    )
    .as_millis() as u32;

    led.set_low();
    relay.set_high();

    ufmt::uwriteln!(&mut serial, "Starting laser exhaust control.\n\nSettings:\n  - Initial ignore period: {}\n  - Shutdown delay: {}\n\r", initial_ignore_period, exhaust_runtime).unwrap_infallible();

    delay_ms(initial_ignore_period);

    // Setup timer0 for 16ms interrupts.
    dp.TC0.tccr0a.write(|w| w.wgm0().ctc());
    dp.TC0.ocr0a.write(|w| w.bits(250));
    dp.TC0.tccr0b.write(|w| w.cs0().prescale_1024());

    // Configure INT0 on pin 2 for raising and falling edge detection.
    dp.EXINT.eicra.modify(|_, w| w.isc0().bits(0b01));
    // Enable the INT0 interrupt source.
    dp.EXINT.eimsk.modify(|_, w| w.int0().set_bit());

    unsafe {
        avr_device::interrupt::enable();
    }

    loop {
        arduino_hal::delay_ms(100);
        avr_device::interrupt::free(|cs| {
            let pin_interrupt_triggered_cell = PIN_INTERRUPT_TRIGGERED.borrow(cs);
            let current_state_cell = CURRENT_STATE.borrow(cs);
            let remaining_time_cell = REMAINING_RUNTIME.borrow(cs);
            match (
                pin_interrupt_triggered_cell.get(),
                interrupt_pin.is_high(),
                current_state_cell.get(),
                remaining_time_cell.get(),
            ) {
                (true, true, State::Idle | State::Cooldown, _) => {
                    ufmt::uwriteln!(&mut serial, "Laser on\r").unwrap_infallible();
                    led.set_high();
                    relay.set_low();
                    dp.TC0.timsk0.write(|w| w.ocie0a().clear_bit());
                    current_state_cell.set(State::Lasing);
                    pin_interrupt_triggered_cell.set(false);
                }
                (true, false, State::Lasing, _) => {
                    ufmt::uwriteln!(&mut serial, "Laser off, cooldown\r").unwrap_infallible();
                    remaining_time_cell.set(exhaust_runtime);
                    dp.TC0.timsk0.write(|w| w.ocie0a().set_bit());
                    current_state_cell.set(State::Cooldown);
                    pin_interrupt_triggered_cell.set(false);
                }
                (true, _, _, _) => {
                    // Interrupt triggered while in an invalid state, ignore.
                    pin_interrupt_triggered_cell.set(false);
                }
                (false, _, State::Cooldown, 0) => {
                    ufmt::uwriteln!(&mut serial, "Cooldown done\r").unwrap_infallible();
                    led.set_low();
                    relay.set_high();
                    dp.TC0.timsk0.write(|w| w.ocie0a().clear_bit());
                    current_state_cell.set(State::Idle);
                }
                (false, _, State::Cooldown, _) => {
                    // Blink during cooldown
                    led.toggle();
                }
                _ => {}
            }
        });
    }
}
