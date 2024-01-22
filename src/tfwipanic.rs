use core::panic::PanicInfo;
use cortex_m::asm::nop;
use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::Peripherals;
use embassy_time::{Duration, block_for};
use stm32_fmc::reset_reg;

// #[panic_handler]
// fn panic(info: &PanicInfo) -> ! {
//     info!("nucleo panicked!!!!!");
//     loop {
//     }
// }
// // Custom panic behavior goes here
// // let p = Peripherals::take_with_cs(cs);
//
// // let p = cortex_m::peripheral::Peripherals::steal().;
//
// let mut red_led = Output::new(p., Level::Low, Speed::VeryHigh);
// let mut blue_led = Output::new(p.PB7, Level::Low, Speed::VeryHigh);
// let mut green_led = Output::new(p.PB0, Level::Low, Speed::VeryHigh);
// let mut button = ExtiInput::new(Input::new(p.PC13, Pull::Down), p.EXTI13);
//
//
// loop {
//     // embedded systems enter an infinite loop on panic
//     red_led.set_high();
//     block_for(Duration::from_millis(500));
//     blue_led.set_high();
//     block_for(Duration::from_millis(500));
//     green_led.set_high();
//     block_for(Duration::from_millis(500));
//     red_led.set_low();
//     blue_led.set_low();
//     green_led.set_low();
//     block_for(Duration::from_millis(500));
//     red_led.set_high();
//     block_for(Duration::from_millis(100));
//     red_led.set_low();
//     block_for(Duration::from_millis(100));
//     red_led.set_high();
//     block_for(Duration::from_millis(100));
//     red_led.set_low();
//     block_for(Duration::from_millis(500));
// }