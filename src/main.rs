// #![feature(trait_alias)]
#![feature(type_alias_impl_trait)]
#![no_std]
#![no_main]

mod tfwipanic;

use core::cell::UnsafeCell;
use cortex_m::interrupt::Mutex;
use defmt::export::panic;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Address, Stack, StackResources};
use embassy_net::tcp::TcpSocket;
use embassy_stm32::{bind_interrupts, eth, peripherals, rng, Config};
use embassy_stm32::eth::{Ethernet, PacketQueue};
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
use embassy_stm32::peripherals::ETH;
use embedded_nal_async::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpConnect};
use rand_core::RngCore;

// static RED_LED: Mutex<UnsafeCell<Option<Output<'static, Flex<dyn Pin<ExtiChannel=(), P=()>>>>>> = Mutex::new(UnsafeCell::new(None));

bind_interrupts!(struct Irqs {
    ETH => eth::InterruptHandler;
    RNG => rng::InterruptHandler<peripherals::RNG>;
});

type Device = Ethernet<'static, embassy_stm32::peripherals::ETH, GenericSMI>;

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<Device>) -> ! {
    stack.run().await
}


#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hsi = Some(HSIPrescaler::DIV1);
        config.rcc.csi = true;
        config.rcc.hsi48 = Some(Default::default()); // needed for RNG
        config.rcc.pll1 = Some(Pll {
            source: PllSource::HSI,
            prediv: PllPreDiv::DIV4,
            mul: PllMul::MUL50,
            divp: Some(PllDiv::DIV2),
            divq: None,
            divr: None,
        });
        config.rcc.sys = Sysclk::PLL1_P; // 400 Mhz
        config.rcc.ahb_pre = AHBPrescaler::DIV2; // 200 Mhz
        config.rcc.apb1_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb2_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb3_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.apb4_pre = APBPrescaler::DIV2; // 100 Mhz
        config.rcc.voltage_scale = VoltageScale::Scale1;
    }
    let p = embassy_stm32::init(config);
    let mut red_led = Output::new(p.PB14, Level::Low, Speed::VeryHigh);
    let mut blue_led = Output::new(p.PB7, Level::Low, Speed::VeryHigh);
    let mut green_led = Output::new(p.PB0, Level::Low, Speed::VeryHigh);
    let mut button = ExtiInput::new(Input::new(p.PC13, Pull::Down), p.EXTI13);

    let mac_addr = [0x00, 0x00, 0x00, 0x69, 0x42, 0x00];

    let device = Ethernet::new(
        make_static!(PacketQueue::<16, 16>::new()), p.ETH, Irqs, p.PA1, p.PA2, p.PC1, p.PA7, p.PC4, p.PC5, p.PG13,
        p.PB13, p.PG11, GenericSMI::new(0), mac_addr,
    );




    let config = embassy_net::Config::dhcpv4(Default::default());
    let mut rng = Rng::new(p.RNG, Irqs);
    let mut seed = [0; 8];
    rng.fill_bytes(&mut seed);
    let seed = u64::from_le_bytes(seed);
    let stack = &*make_static!(Stack::new(
        device,
        config,
        make_static!(StackResources::<3>::new()),
        seed
    ));

    unwrap!(spawner.spawn(net_task(&stack)));


    stack.wait_config_up().await;

    info!("Network task initialized");

    let mut rx_buffer = [0; 1024];
    let mut tx_buffer = [0; 1024];


    loop {
        // there needs to be a server running on the ground station on the below (static) ip
        let remote_endpoint = (Ipv4Address::new(192, 168, 1, 4), 6942);
        let mut socket = TcpSocket::new(&stack, &mut rx_buffer, &mut tx_buffer);

        socket.set_timeout(Some(embassy_time::Duration::from_secs(10)));

        info!("connecting...");
        let r = socket.connect(remote_endpoint).await;
        if let Err(e) = r {
            info!("connect error: {:?}", e);
            Timer::after_secs(1).await;
            continue;
        }
        info!("connected!");
        loop {
            let r = socket.write_all(b"Hello World.\n").await;
            if let Err(e) = r {
                info!("write error: {:?}", e);
                break;
            }
            Timer::after_secs(1).await;
        }
    }
}
