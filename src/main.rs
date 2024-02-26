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
use embassy_stm32::can::frame::{ClassicData, FdFrame, Header};
use embassy_stm32::peripherals::{ETH, FDCAN1};
use embedded_nal_async::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpConnect};
use rand_core::RngCore;
use embedded_can;
// use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("\x1b[106;31m hello world! \x1b[0m");

    let mut config = Config::default();
    config.rcc.hse = Some(rcc::Hse {
        freq: embassy_stm32::time::Hertz(8_000_000),
        mode: rcc::HseMode::Bypass,
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

    let mut can = can.into_normal_mode();
    info!("\x1b[106;34m checkpoint 6 \x1b[0m");


    info!("Configured");


    let mut green_led = Output::new(peripherals.PB0, Level::High, Speed::Low);
    let mut i = 1u8;
    let mut last_read_ts = embassy_time::Instant::now();


    info!("entering loop: writing");
    loop {
        // let id = embedded_can::Id::Standard(embedded_can::StandardId::new(i as u16).unwrap());
        // let payload = ClassicData::new(&[0, 0, 0]).unwrap();
        // let frame = can::frame::ClassicFrame::new(Header::new(id, 8u8, false), payload);
        let frame = can::frame::ClassicFrame::new_standard(i as u16, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
        info!("Writing frame id={}", i);
        _ = can.write(&frame).await;
        i+=1;
        can.flush(0usize);
        Timer::after_millis(1500).await;
    }
    //
    // info!("entering loop: reading");
    // loop {
    //     match can.read_fd().await {
    //         Ok((fd_frame, ts)) => {
    //             let delta = (ts - last_read_ts).as_millis();
    //             last_read_ts = ts;
    //             info!(
    //                 "Rx: {:x} {:x} {:x} {:x} --- NEW {}",
    //                 fd_frame.data()[0],
    //                 fd_frame.data()[1],
    //                 fd_frame.data()[2],
    //                 fd_frame.data()[3],
    //                 delta,
    //             )
    //         }
    //         Err(e) => {
    //             error!("something broke :(");
    //             // error!("{}",e.to_string());
    //         }
    //     }
    // }
}
