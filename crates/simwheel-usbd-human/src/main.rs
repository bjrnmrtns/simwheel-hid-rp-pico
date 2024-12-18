#![no_std]
#![no_main]

use bsp::entry;
use bsp::hal;
use cortex_m::prelude::*;
use defmt::*;
use defmt_rtt as _;
use embedded_hal::digital::*;
use fugit::ExtU32;
use hal::{
    gpio::{DynPinId, FunctionSioInput, Pin, PullUp},
    pac,
};
use panic_probe as _;
#[allow(clippy::wildcard_imports)]
use usb_device::class_prelude::*;
use usb_device::prelude::*;
use usbd_human_interface_device::device::joystick::JoystickReport;
use usbd_human_interface_device::prelude::*;

use rp_pico as bsp;

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let timer = hal::Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

    let sio = hal::Sio::new(pac.SIO);
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    info!("Starting");

    //USB
    let usb_bus = UsbBusAllocator::new(hal::usb::UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut pac.RESETS,
    ));

    let mut joy = UsbHidClassBuilder::new()
        .add_device(usbd_human_interface_device::device::joystick::JoystickConfig::default())
        .build(&usb_bus);

    //https://pid.codes
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x1209, 0x0001))
        .strings(&[StringDescriptors::default()
            .manufacturer("usbd-human-interface-device")
            .product("Rusty joystick")
            .serial_number("TEST")])
        .unwrap()
        .build();

    //GPIO pins
    let mut input_pins: [Pin<DynPinId, FunctionSioInput, PullUp>; 8] = [
        pins.gpio0.into_pull_up_input().into_dyn_pin(),
        pins.gpio1.into_pull_up_input().into_dyn_pin(),
        pins.gpio2.into_pull_up_input().into_dyn_pin(),
        pins.gpio3.into_pull_up_input().into_dyn_pin(),
        pins.gpio19.into_pull_up_input().into_dyn_pin(),
        pins.gpio20.into_pull_up_input().into_dyn_pin(),
        pins.gpio21.into_pull_up_input().into_dyn_pin(),
        pins.gpio22.into_pull_up_input().into_dyn_pin(),
    ];

    let mut input_count_down = timer.count_down();
    input_count_down.start(10.millis());

    loop {
        // Poll every 10ms
        if input_count_down.wait().is_ok() {
            match joy.device().write_report(&get_report(&mut input_pins)) {
                Err(UsbHidError::WouldBlock) => {}
                Ok(_) => {}
                Err(e) => {
                    core::panic!("Failed to write joystick report: {:?}", e)
                }
            }
        }

        if usb_dev.poll(&mut [&mut joy]) {}
    }
}

fn get_report(pins: &mut [Pin<DynPinId, FunctionSioInput, PullUp>; 8]) -> JoystickReport {
    // Read out 8 buttons first
    let mut buttons = 0;
    for (idx, pin) in pins[0..8].iter_mut().enumerate() {
        if pin.is_low().unwrap() {
            buttons |= 1 << idx;
        }
    }
/*
    // We're using digital switches in a D-PAD style configuration
    //    10
    //  8    9
    //    11
    // These are mapped to the limits of an axis
    let x = if pins[8].is_low().unwrap() {
        -127 // left
    } else if pins[9].is_low().unwrap() {
        127 // right
    } else {
        0 // center
    };

    let y = if pins[10].is_low().unwrap() {
        -127 // up
    } else if pins[11].is_low().unwrap() {
        127 // down
    } else {
        0 // center
    };
*/

    JoystickReport { buttons, x: 0, y: 0 }
}
