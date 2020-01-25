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
use lib::hal::stm32::{self, interrupt, TIM2, TIM3};
use lib::hal::timer::{Event as TimerEvent, Timer};
use lib::logger::Logger;
use lib::net::eth::{Eth, MTU, NEIGHBOR_CACHE_SIZE, SOCKET_BUFFER_SIZE};
use lib::params::{pop_event, push_event, Params};
use lib::sys_clock;
use log::{debug, info, warn, LevelFilter};
use param_desc::{node_id::TEMPLATE_NODE1, param, param_id};
use params::{
    GetSetFlags, GetSetFrame, GetSetNodeId, GetSetOp, GetSetPayloadType, Parameter, ParameterValue,
    RefResponse, Request, Response, PREAMBLE_WORD,
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

const TCP_SERVER_IP: Ipv4Address = Ipv4Address(SRC_IP);
const TCP_SERVER_PORT: u16 = 9877;

const NODE_ID: GetSetNodeId = TEMPLATE_NODE1;

const PARAMETERS: [&'static Parameter; 4] = [
    &param::UPTIME,
    &param::ETH_LINK_DOWN_COUNT,
    &param::LED_STATE,
    &param::TEMPERATURE,
];

static GLOBAL_LOGGER: Logger = Logger::new();

static GLOBAL_ETH_PENDING: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

static GLOBAL_PARAM_BCAST_TIM2: Mutex<RefCell<Option<Timer<TIM2>>>> =
    Mutex::new(RefCell::new(None));
static GLOBAL_PARAM_BCAST_PENDING: Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

static GLOBAL_ETH_POLL_TIM3: Mutex<RefCell<Option<Timer<TIM3>>>> = Mutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let dp = stm32::Peripherals::take().expect("Failed to take stm32::Peripherals");
    let mut cp =
        cortex_m::peripheral::Peripherals::take().expect("Failed to take cortex_m::Peripherals");

    stm32_eth::setup(&dp.RCC, &dp.SYSCFG);

    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(180.mhz()).freeze();

    let gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();
    let gpioc = dp.GPIOC.split();
    let gpiod = dp.GPIOD.split();
    let gpiog = dp.GPIOG.split();

    // LEDs, turn blue on during setup
    let mut led_green = gpiob.pb0.into_push_pull_output();
    let mut led_blue = gpiob.pb7.into_push_pull_output();
    let mut led_red = gpiob.pb14.into_push_pull_output();
    led_green.set_low().unwrap();
    led_blue.set_high().unwrap();
    led_red.set_low().unwrap();

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

    debug!("Setup system clock");
    sys_clock::start(cp.SYST, clocks);

    debug!("Setup parameters");
    let mut params = Params::new();
    for p in &PARAMETERS {
        params.add(**p).unwrap();
    }

    debug!("Setup Ethernet");
    stm32_eth::setup_pins(
        gpioa.pa1, gpioa.pa2, gpioa.pa7, gpiob.pb13, gpioc.pc1, gpioc.pc4, gpioc.pc5, gpiog.pg11,
        gpiog.pg13,
    );

    let mut rx_ring: [stm32_eth::RingEntry<_>; 16] = Default::default();
    let mut tx_ring: [stm32_eth::RingEntry<_>; 8] = Default::default();
    let mut stm_eth = stm32_eth::Eth::new(
        dp.ETHERNET_MAC,
        dp.ETHERNET_DMA,
        SRC_MAC,
        &mut rx_ring[..],
        &mut tx_ring[..],
    );
    stm_eth.enable_interrupt(&mut cp.NVIC);

    debug!("Link: {}", stm_eth.status().link_detected());

    debug!("Setup IP stack");
    assert_eq!((&mut stm_eth).capabilities().max_transmission_unit, MTU);
    let ip = Ipv4Address::from_bytes(&SRC_IP);
    let mac = EthernetAddress::from_bytes(&SRC_MAC);
    info!("IP: {} MAC: {}", ip, mac);
    let ip_addr = IpCidr::new(ip.into(), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; NEIGHBOR_CACHE_SIZE];
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut routes_storage = [];
    let routes = Routes::new(&mut routes_storage[..]);
    let iface = EthernetInterfaceBuilder::new(&mut stm_eth)
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

    // General purpose eth frame buffer
    let eth_frame_buffer = {
        static mut BUFFER: [u8; MTU] = [0; MTU];
        unsafe { &mut BUFFER[..] }
    };

    let tcp_handle = sockets.add(tcp_socket);
    let udp_handle = sockets.add(udp_socket);

    let tcp_endpoint = IpEndpoint::new(TCP_SERVER_IP.into(), TCP_SERVER_PORT);
    let udp_endpoint = IpEndpoint::new(UDP_BCAST_IP.into(), UDP_BCAST_PORT);

    let mut eth = Eth::new(
        iface,
        sockets,
        tcp_handle,
        tcp_endpoint,
        udp_handle,
        udp_endpoint,
    )
    .unwrap();

    debug!("Setup timers");
    let mut param_bcast_timer = Timer::tim2(dp.TIM2, 1.hz(), clocks);
    let mut eth_poll_timer = Timer::tim3(dp.TIM3, 20.hz(), clocks);
    param_bcast_timer.listen(TimerEvent::TimeOut);
    eth_poll_timer.listen(TimerEvent::TimeOut);

    cortex_m::interrupt::free(|cs| {
        GLOBAL_PARAM_BCAST_TIM2
            .borrow(cs)
            .replace(Some(param_bcast_timer));
        GLOBAL_ETH_POLL_TIM3
            .borrow(cs)
            .replace(Some(eth_poll_timer));
    });

    // Enable timer interrupts
    stm32::NVIC::unpend(interrupt::TIM2);
    unsafe {
        stm32::NVIC::unmask(interrupt::TIM2);
    };
    stm32::NVIC::unpend(interrupt::TIM3);
    unsafe {
        stm32::NVIC::unmask(interrupt::TIM3);
    };

    // Handle initial setup from params
    match params.get_value(param_id::LED_STATE).unwrap().as_bool() {
        true => led_red.set_high().unwrap(),
        false => led_red.set_low().unwrap(),
    }

    // TODO - sometimes UDP broadcast data doesn't get recv'd on my host?
    // have to reset the board
    // not sure if it's the board or my network/router/etc
    //
    // - blink LED with status?
    // - manage eth.status() (PhyStatus) events
    // - eth.get_phy() -> Phy, can reset/etc
    // - wait for link to be up?
    //
    // setup watchdog and parameter to hold last reset condition
    // make them read-only
    //
    // make a path for forcing bcast to pending when something changes?
    // then no rate limit exist?
    led_blue.set_low().unwrap();
    let mut last_sec = 0;
    loop {
        let time = sys_clock::system_time();

        // TODO - error handling
        let param_bcast_pending =
            cortex_m::interrupt::free(|cs| GLOBAL_PARAM_BCAST_PENDING.borrow(cs).replace(false));
        if param_bcast_pending {
            let mut frame = GetSetFrame::new_unchecked(&mut eth_frame_buffer[..]);
            let bcast_params = params.get_all_broadcast();
            if bcast_params.len() != 0 {
                let ref_resp =
                    RefResponse::new(NODE_ID, GetSetFlags::default(), GetSetOp::Get, bcast_params);
                ref_resp.emit(&mut frame).unwrap();
                let size = ref_resp.wire_size();
                eth.send_udp_bcast(&frame.as_ref()[..size]).unwrap();
            }
        }

        // Set by Eth interrupt and polling timer interrupt
        let eth_pending =
            cortex_m::interrupt::free(|cs| GLOBAL_ETH_PENDING.borrow(cs).replace(false));
        if eth_pending || param_bcast_pending {
            eth.poll(time);
        }

        // TODO - error handling
        // Service TCP server
        if let Ok(bytes_recvd) = eth.recv_tcp_frame(&mut eth_frame_buffer[..]) {
            if let Ok(frame) = GetSetFrame::new_checked(&eth_frame_buffer[..bytes_recvd]) {
                cortex_m::interrupt::free(|cs| GLOBAL_ETH_PENDING.borrow(cs).replace(true));
                debug!("Rx {}", frame);
                match frame.op() {
                    GetSetOp::ListAll => {
                        let mut frame = GetSetFrame::new_unchecked(&mut eth_frame_buffer[..]);
                        let params = params.as_ref();
                        if params.len() != 0 {
                            let ref_resp = RefResponse::new(
                                NODE_ID,
                                GetSetFlags::default(),
                                GetSetOp::ListAll,
                                params,
                            );
                            ref_resp.emit(&mut frame).unwrap();
                            debug!("Tx {}", frame);
                            let size = ref_resp.wire_size();
                            eth.send_tcp(&frame.as_ref()[..size]).unwrap();
                        }
                    }
                    GetSetOp::Get
                        if frame.payload_type() == GetSetPayloadType::ParameterIdListPacket =>
                    {
                        let req = Request::parse(&frame).unwrap();
                        let mut resp =
                            Response::new(NODE_ID, GetSetFlags::default(), GetSetOp::Get);
                        for id in req.ids() {
                            if let Some(p) = params.get(*id) {
                                resp.push(*p).unwrap();
                            }
                        }

                        let mut frame = GetSetFrame::new_unchecked(&mut eth_frame_buffer[..]);
                        resp.emit(&mut frame).unwrap();
                        debug!("Tx {}", frame);
                        let size = resp.wire_size();
                        eth.send_tcp(&frame.as_ref()[..size]).unwrap();
                    }
                    GetSetOp::Set
                        if frame.payload_type() == GetSetPayloadType::ParameterListPacket =>
                    {
                        let req = Request::parse(&frame).unwrap();
                        let mut resp =
                            Response::new(NODE_ID, GetSetFlags::default(), GetSetOp::Set);
                        for p in req.parameters() {
                            // TODO - callback notification in here somewhere?
                            if params.set(p.id(), p.value(), false).is_ok() {
                                resp.push(*params.get(p.id()).unwrap()).unwrap();

                                match p.id() {
                                    param_id::LED_STATE => match p.value().as_bool() {
                                        true => led_red.set_high().unwrap(),
                                        false => led_red.set_low().unwrap(),
                                    },
                                    _ => (),
                                }
                            }
                        }

                        let mut frame = GetSetFrame::new_unchecked(&mut eth_frame_buffer[..]);
                        resp.emit(&mut frame).unwrap();
                        debug!("Tx {}", frame);
                        let size = resp.wire_size();
                        eth.send_tcp(&frame.as_ref()[..size]).unwrap();
                    }
                    op @ _ => {
                        warn!("Got malformed request {}", frame);
                        let mut frame = GetSetFrame::new_unchecked(&mut eth_frame_buffer[..]);
                        frame.set_preamble(PREAMBLE_WORD);
                        frame.set_node_id(NODE_ID);
                        frame.set_flags(GetSetFlags::default());
                        frame.set_version(1);
                        frame.set_op(op);
                        frame.set_payload_type(GetSetPayloadType::None);
                        frame.set_payload_size(0);
                        debug!("Tx {}", frame);
                        let size = GetSetFrame::<&[u8]>::header_len();
                        eth.send_tcp(&frame.as_ref()[..size]).unwrap();
                    }
                }
            }
        }

        // Drain parameter event queue
        pop_event().map(|e| params.process_event(e).unwrap());

        let sec = time.as_secs();
        if sec != last_sec {
            last_sec = sec;
            led_green.toggle().unwrap();

            // TODO
            let inner = params.get_value(param_id::UPTIME).unwrap().as_u32();
            push_event((param_id::UPTIME, ParameterValue::U32(inner.wrapping_add(1))).into()).ok();

            let inner = params.get_value(param_id::TEMPERATURE).unwrap().as_f32();
            push_event((param_id::TEMPERATURE, ParameterValue::F32(inner + 0.13)).into()).ok();
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

#[interrupt]
fn TIM3() {
    cortex_m::interrupt::free(|cs| {
        if let Some(ref mut tim) = GLOBAL_ETH_POLL_TIM3.borrow(cs).borrow_mut().deref_mut() {
            tim.clear_interrupt(TimerEvent::TimeOut);
            GLOBAL_ETH_PENDING.borrow(cs).replace(true);
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
