// #![feature(trait_alias)]
#![allow(unused)]
#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

// mod tfwipanic;

use core::cell::UnsafeCell;
use cortex_m::interrupt::Mutex;
use defmt::export::panic;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Stack, StackResources};
use embassy_net::tcp::TcpSocket;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng, Config, can, rcc};
use embassy_stm32::eth::{Ethernet, InterruptHandler, PacketQueue};
use embassy_stm32::eth::generic_smi::GenericSMI;
use embassy_stm32::exti::ExtiInput;
use embassy_stm32::gpio::{Flex, Input, Level, Output, Pin, Pull, Speed};
use embassy_stm32::rng::Rng;
use {defmt_rtt as _, panic_probe as _};
use embassy_time::{Duration, block_for, Timer};
use static_cell::make_static;
use embedded_io_async::Write;
use defmt::*;
use embassy_net::tcp::client::{TcpClient, TcpClientState};
use embassy_stm32::peripherals::{ETH, FDCAN1};
use embedded_nal_async::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpConnect};
use rand_core::RngCore;

use static_cell::StaticCell;

// static RED_LED: Mutex<UnsafeCell<Option<Output<'static, Flex<dyn Pin<ExtiChannel=(), P=()>>>>>> = Mutex::new(UnsafeCell::new(None));

// bind_interrupts!(struct Irqs {
//     ETH => eth::InterruptHandler;
//     RNG => rng::InterruptHandler<peripherals::RNG>;
// });
// bind_interrupts!(struct Irqs {
//     FDCAN1_IT0 => ;
//     // CAN3_RX1 => Rx1InterruptHandler<CAN3>;
//     // CAN3_SCE => SceInterruptHandler<CAN3>;
//     // CAN3_TX => TxInterruptHandler<CAN3>;
// });

// type Device = Ethernet<'static, embassy_stm32::peripherals::ETH, GenericSMI>;

// #[embassy_executor::task]
// async fn net_task(stack: &'static Stack<Device>) -> ! {
//     stack.run().await
// }
bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("\x1b[106;31m hello world! \x1b[0m");

    // config.rcc.fdcan_clock_source = rcc::FdCanClockSource::HSE;
    //
    // let peripherals = embassy_stm32::init(config);
    //
    // let mut can = can::Fdcan::new(peripherals.FDCAN1, peripherals.PA11, peripherals.PA12, Irqs);
    //
    // can.can.apply_config(
    //     can::config::FdCanConfig::default().set_nominal_bit_timing(can::config::NominalBitTiming {
    //         sync_jump_width: 1.try_into().unwrap(),
    //         prescaler: 8.try_into().unwrap(),
    //         seg1: 13.try_into().unwrap(),
    //         seg2: 2.try_into().unwrap(),
    //     }),
    // );
    let mut config = Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(25_000_000),
        mode: rcc::HseMode::Oscillator,
    });
    info!("\x1b[106;34m checkpoint 1 \x1b[0m");
    config.rcc.fdcan_clock_source = rcc::FdCanClockSource::HSE;
    info!("\x1b[106;34m checkpoint 2 \x1b[0m");

    let peripherals = embassy_stm32::init(config);
    info!("\x1b[106;34m checkpoint 3 \x1b[0m");

    let mut can = can::FdcanConfigurator::new(peripherals.FDCAN1, peripherals.PD0, peripherals.PD1, Irqs);
    info!("\x1b[106;34m checkpoint 4 \x1b[0m");

    // 250k bps
    can.set_bitrate(250_000);
    info!("\x1b[106;34m checkpoint 5 \x1b[0m");

    // let mut can = can.into_internal_loopback_mode();
    // let mut can = can.into_internal_loopback_mode();
    let mut can = can.into_normal_mode();
    // let mut can = can.start(can::FdcanOperatingMode::BusMonitoringMode);
    info!("\x1b[106;34m checkpoint 6 \x1b[0m");


    info!("Configured");


    let mut green_led = Output::new(peripherals.PB0, Level::High, Speed::Low);
    let mut i = 0;
    let mut last_read_ts = embassy_time::Instant::now();

    info!("entering loop");
    loop {
        let frame = can::frame::ClassicFrame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame");
        _ = can.write(&frame).await;
        info!("\x1b[106;34m checkpoint 7 \x1b[0m");

        match can.read().await {
            Ok((rx_frame, ts)) => {
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {:x} {:x} {:x} {:x} --- NEW {}",
                    rx_frame.data()[0],
                    rx_frame.data()[1],
                    rx_frame.data()[2],
                    rx_frame.data()[3],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        info!("\x1b[106;34m checkpoint 8 \x1b[0m");

        Timer::after_millis(250).await;

        i += 1;
        if i > 3 {
            break;
        }
    }

    info!("\x1b[106;34m checkpoint 9: finished first loop \x1b[0m");


    let (mut tx, mut rx) = can.split();
    // With split
    loop {
        let frame = can::frame::ClassicFrame::new_extended(0x123456F, &[i; 8]).unwrap();
        info!("Writing frame with split");
        _ = tx.write(&frame).await;
        info!("\x1b[106;34m checkpoint 10: split sent \x1b[0m");

        match rx.read().await {
            Ok((rx_frame, ts)) => {
                let delta = (ts - last_read_ts).as_millis();
                last_read_ts = ts;
                info!(
                    "Rx: {:x} {:x} {:x} {:x} --- NEW {}",
                    rx_frame.data()[0],
                    rx_frame.data()[1],
                    rx_frame.data()[2],
                    rx_frame.data()[3],
                    delta,
                )
            }
            Err(_err) => error!("Error in frame"),
        }

        Timer::after_millis(250).await;

        i += 1;
    }
}
