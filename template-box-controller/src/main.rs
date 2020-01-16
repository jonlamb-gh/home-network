#![no_std]
#![no_main]
#![feature(core_intrinsics)]

use core::cell::{Cell, RefCell};
use core::ops::DerefMut;
use cortex_m::interrupt::Mutex;
use cortex_m_rt::ExceptionFrame;
use cortex_m_rt::{entry, exception};
use lib::hal::prelude::*;
use lib::hal::serial::{config::Config, Serial};
use lib::hal::stm32::{self, interrupt, TIM2};
use lib::hal::timer::{Event as TimerEvent, Timer};
use lib::logger::Logger;
use lib::net::eth::{Eth, MTU, NEIGHBOR_CACHE_SIZE, SOCKET_BUFFER_SIZE};
use lib::params::Params;
use lib::sys_clock;
use log::{debug, info, LevelFilter};
use params::{
    GetSetFrame, GetSetOp, Parameter, ParameterFlags, ParameterId, ParameterValue, RefResponse,
};
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::phy::Device;
use smoltcp::socket::{
    SocketSet, TcpSocket, TcpSocketBuffer, UdpPacketMetadata, UdpSocket, UdpSocketBuffer,
};
use smoltcp::wire::{EthernetAddress, IpCidr, IpEndpoint, Ipv4Address};

mod panic_handler;

const SRC_MAC: [u8; 6] = [0x02, 0x00, 0x05, 0x06, 0x07, 0x08];
const SRC_IP: [u8; 4] = [192, 168, 1, 39];

const UDP_BCAST_IP: Ipv4Address = Ipv4Address::BROADCAST;
const UDP_BCAST_PORT: u16 = 9876;

// Not sure having these make sense?
const STATIC_RO_PARAMS: [Parameter; 6] = [
    Parameter::new_with_value(
        ParameterId::new(1),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::Notification,
    ),
    Parameter::new_with_value(
        ParameterId::new(2),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::Bool(false),
    ),
    Parameter::new_with_value(
        ParameterId::new(3),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::U8(123),
    ),
    Parameter::new_with_value(
        ParameterId::new(4),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::U32(3456),
    ),
    Parameter::new_with_value(
        ParameterId::new(5),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::I32(-123),
    ),
    Parameter::new_with_value(
        ParameterId::new(6),
        ParameterFlags::new_read_only_broadcast(),
        ParameterValue::F32(-1.234),
    ),
];

static GLOBAL_LOGGER: Logger = Logger::new();

static GLOBAL_ETH_PENDING: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

static GLOBAL_PARAM_BCAST_TIM2: Mutex<RefCell<Option<Timer<TIM2>>>> =
    Mutex::new(RefCell::new(None));

static GLOBAL_PARAM_BCAST_PENDING: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let mut cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    stm32_eth::setup(&dp.RCC, &dp.SYSCFG);

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

    let gpiod = dp.GPIOD.split();
    let pin_tx = gpiod.pd8.into_alternate_af7();
    let pin_rx = gpiod.pd9.into_alternate_af7();

    let serial = Serial::usart3(
        dp.USART3,
        (pin_tx, pin_rx),
        Config {
            baudrate: 115_200.bps(),
            ..Default::default()
        },
        clocks,
    )
    .unwrap();

    // Setup logger on USART3
    let (tx, _rx) = serial.split();
    GLOBAL_LOGGER.set_inner(tx);
    log::set_logger(&GLOBAL_LOGGER).unwrap();
    log::set_max_level(LevelFilter::Trace);

    // TODO - impl core::fmt:Display for things
    debug!("Setup parameters");
    let mut params = Params::new();
    for p in &STATIC_RO_PARAMS {
        debug!("Adding parameter ID {:?}", p.id());
        params.add(*p).unwrap();
    }

    debug!("Setup Ethernet");
    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiog = dp.GPIOG.split();
    stm32_eth::setup_pins(
        gpioa.pa1, gpioa.pa2, gpioa.pa7, gpiob.pb13, gpioc.pc1, gpioc.pc4, gpioc.pc5, gpiog.pg11,
        gpiog.pg13,
    );

    let mut rx_ring: [stm32_eth::RingEntry<_>; 16] = Default::default();
    let mut tx_ring: [stm32_eth::RingEntry<_>; 8] = Default::default();
    let mut eth = stm32_eth::Eth::new(
        dp.ETHERNET_MAC,
        dp.ETHERNET_DMA,
        SRC_MAC,
        &mut rx_ring[..],
        &mut tx_ring[..],
    );
    eth.enable_interrupt(&mut cp.NVIC);

    debug!("Setup IP stack");
    assert_eq!((&mut eth).capabilities().max_transmission_unit, MTU);
    let ip = Ipv4Address::from_bytes(&SRC_IP);
    let mac = EthernetAddress::from_bytes(&SRC_MAC);
    info!("IP: {} MAC: {}", ip, mac);
    let ip_addr = IpCidr::new(ip.into(), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; NEIGHBOR_CACHE_SIZE];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut routes_storage = [];
    let routes = Routes::new(&mut routes_storage[..]);
    let iface = EthernetInterfaceBuilder::new(&mut eth)
        .ethernet_addr(mac.into())
        .ip_addrs(&mut ip_addrs[..])
        .neighbor_cache(neighbor_cache)
        .routes(routes)
        .finalize();

    let mut sockets_storage = [None, None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);

    let tcp_socket = {
        static mut RX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        static mut TX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        TcpSocket::new(
            TcpSocketBuffer::new(unsafe { &mut RX_BUFFER[..] }),
            TcpSocketBuffer::new(unsafe { &mut TX_BUFFER[..] }),
        )
    };

    let mut rx_meta = [UdpPacketMetadata::EMPTY];
    let mut tx_meta = [UdpPacketMetadata::EMPTY];
    let udp_socket = {
        static mut RX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        static mut TX_BUFFER: [u8; SOCKET_BUFFER_SIZE] = [0; SOCKET_BUFFER_SIZE];
        UdpSocket::new(
            UdpSocketBuffer::new(&mut rx_meta[..], unsafe { &mut RX_BUFFER[..] }),
            UdpSocketBuffer::new(&mut tx_meta[..], unsafe { &mut TX_BUFFER[..] }),
        )
    };

    // Parameter response buffer
    let param_resp_buffer = {
        static mut BUFFER: [u8; MTU] = [0; MTU];
        unsafe { &mut BUFFER[..] }
    };

    let tcp_handle = sockets.add(tcp_socket);
    let udp_handle = sockets.add(udp_socket);

    let udp_endpoint = IpEndpoint::new(UDP_BCAST_IP.into(), UDP_BCAST_PORT);

    // TODO - manage UDP broadcast and TCP sockets in ::eth
    // poll returns something we know like request/response/etc
    let mut eth = Eth::new(iface, sockets, tcp_handle, udp_handle, udp_endpoint).unwrap();

    debug!("Setup timers");
    let mut param_bcast_timer = Timer::tim2(dp.TIM2, 1.hz(), clocks);
    param_bcast_timer.listen(TimerEvent::TimeOut);

    cortex_m::interrupt::free(|cs| {
        GLOBAL_PARAM_BCAST_TIM2
            .borrow(cs)
            .replace(Some(param_bcast_timer));
    });

    // Enable timer interrupts
    stm32::NVIC::unpend(interrupt::TIM2);
    unsafe {
        stm32::NVIC::unmask(interrupt::TIM2);
    };

    debug!("Setup system clock");
    sys_clock::start(cp.SYST, clocks);

    let mut last_sec = 0;
    loop {
        //TODO
        // need up update paramer.local_time_ms
        let time = sys_clock::system_time();

        // TODO - error handling
        let param_bcast_pending =
            cortex_m::interrupt::free(|cs| GLOBAL_PARAM_BCAST_PENDING.borrow(cs).replace(false));
        if param_bcast_pending {
            let mut frame = GetSetFrame::new_unchecked(&mut param_resp_buffer[..]);
            let bcast_params = params.get_all_broadcast();
            debug!("bcast ({})", bcast_params.len());
            if bcast_params.len() != 0 {
                let ref_resp = RefResponse::new(GetSetOp::Get, bcast_params);
                ref_resp.emit(&mut frame).unwrap();
                let size = ref_resp.wire_size();
                eth.send_udp_bcast(&frame.as_ref()[..size]).unwrap();
            }
        }

        // TODO - timer for eth polling
        let eth_pending =
            cortex_m::interrupt::free(|cs| GLOBAL_ETH_PENDING.borrow(cs).replace(false));
        if eth_pending || param_bcast_pending {
            eth.poll(time);
        }

        let sec = time.as_secs();
        if sec != last_sec {
            info!("{}", lib::time::DisplayableInstant::from(time));
            last_sec = sec;
        }
    }
}

#[exception]
fn SysTick() {
    cortex_m::interrupt::free(|cs| {
        sys_clock::increment(cs);
    });
}

#[interrupt]
fn ETH() {
    cortex_m::interrupt::free(|cs| {
        GLOBAL_ETH_PENDING.borrow(cs).replace(true);
    });

    // Clear interrupt flags
    let p = unsafe { stm32::Peripherals::steal() };
    stm32_eth::eth_interrupt_handler(&p.ETHERNET_DMA);
}

#[interrupt]
fn TIM2() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut tim) = GLOBAL_PARAM_BCAST_TIM2.borrow(cs).borrow_mut().deref_mut() {
            tim.clear_interrupt(TimerEvent::TimeOut);
            GLOBAL_PARAM_BCAST_PENDING.borrow(cs).replace(true);
        }
    });
}

#[exception]
fn HardFault(ef: &ExceptionFrame) -> ! {
    panic!("HardFault at {:#?}", ef);
}

#[exception]
fn DefaultHandler(irqn: i16) {
    panic!("Unhandled exception (IRQn = {})", irqn);
}
