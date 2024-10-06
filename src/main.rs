#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_time::Instant;
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Config, Handler};
use usbd_hid::descriptor::SerializedDescriptor;
use {defmt_rtt as _, panic_probe as _};

const JOYSTICK_HID_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
    0x09, 0x04,        // Usage (Joystick)
    0xA1, 0x01,        // Collection (Application)
    
    // Buttons
    0x05, 0x09,        //   Usage Page (Button)
    0x19, 0x01,        //   Usage Minimum (Button 1)
    0x29, 0x1E,        //   Usage Maximum (Button 30) (0x1E = 30 in hex)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x01,        //   Logical Maximum (1)
    0x75, 0x01,        //   Report Size (1) - 1 bit per button
    0x95, 0x1E,        //   Report Count (30) - 30 buttons
    0x81, 0x02,        //   Input (Data,Var,Abs)

    // Padding for unused bits (2 bits to make the byte boundary)
    0x75, 0x02,        //   Report Size (2) - Padding
    0x95, 0x01,        //   Report Count (1)
    0x81, 0x03,        //   Input (Const,Var,Abs) - Padding bits (not used)

    0xC0               // End Collection
];

// Define the report that will be sent over USB for the 30-button joystick (no axes)
#[derive(Copy, Clone, Debug)]
pub struct JoystickReport {
    pub buttons: [u8; 4], // 30 buttons require 4 bytes (32 bits total, but only 30 used)
}

impl Default for JoystickReport {
    fn default() -> Self {
        JoystickReport {
            buttons: [0; 4],
        }
    }
}

impl SerializedDescriptor for JoystickReport {
    fn desc() -> &'static [u8] {
        JOYSTICK_HID_DESCRIPTOR
    }
}


bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

struct GpioInfo {
    pub last_time: Instant,
    pub buttons: [u8; 4],
    pub previous_pressed: bool,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());

    let driver = Driver::new(p.USB, Irqs);
    let mut config = Config::new(0xc0de, 0xcafe);
    config.manufacturer = Some("Embassy");
    config.product = Some("HID keyboard example");
    config.serial_number = Some("12345678");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    let mut config_descriptor = [0; 256];
    let mut bos_descriptor = [0; 256];
    let mut msos_descriptor = [0; 256];
    let mut control_buf = [0; 64];
    let mut request_handler = MyRequestHandler {};
    let mut device_handler = MyDeviceHandler::new();

    let mut state = State::new();

    let mut builder = Builder::new(
        driver,
        config,
        &mut config_descriptor,
        &mut bos_descriptor,
        &mut msos_descriptor,
        &mut control_buf,
    );

    builder.handler(&mut device_handler);

    let config = embassy_usb::class::hid::Config {
        report_descriptor: JoystickReport::desc(),
        request_handler: None,
        poll_ms: 1,
        max_packet_size: 64,
    };
    let hid = HidReaderWriter::<_, 1, 8>::new(&mut builder, &mut state, config);

    // Build the builder.
    let mut usb = builder.build();
        
    let usb_fut = usb.run();

    let mut gpio0 = Input::new(p.PIN_18, Pull::Up);
    let mut gpio1 = Input::new(p.PIN_19, Pull::Up);
    gpio0.set_schmitt(true);
    gpio1.set_schmitt(true);

    let (reader, mut writer) = hid.split();

    let now = Instant::now();
    let mut gpio_info = [
        GpioInfo { buttons: [0, 0, 0, 1], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0, 0, 0, 2], last_time: now, previous_pressed: false }
    ];

    // Do stuff with the class!
    let in_fut = async {
        loop {
            let (_, index) = embassy_futures::select::select_array([
                gpio0.wait_for_any_edge(),
                gpio1.wait_for_any_edge(),
            ]).await;

            let pressed = [gpio0.is_low(), gpio1.is_low()];
            gpio_info[index].last_time = Instant::now();

            if gpio_info[index].previous_pressed != pressed[index] {
                let report = if pressed[index] {
                    JoystickReport { buttons: gpio_info[index].buttons }
                } else {
                    JoystickReport { buttons: [0, 0, 0, 0] }
                };
                match writer.write(&report.buttons).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                };
            }
            gpio_info[index].previous_pressed = pressed[index];
        }
    };

    let out_fut = async {
        reader.run(false, &mut request_handler).await;
    };

    join(usb_fut, join(in_fut, out_fut)).await;
}

struct MyRequestHandler {}

impl RequestHandler for MyRequestHandler {
    fn get_report(&mut self, id: ReportId, _buf: &mut [u8]) -> Option<usize> {
        info!("Get report for {:?}", id);
        None
    }

    fn set_report(&mut self, id: ReportId, data: &[u8]) -> OutResponse {
        info!("Set report for {:?}: {=[u8]}", id, data);
        OutResponse::Accepted
    }

    fn set_idle_ms(&mut self, id: Option<ReportId>, dur: u32) {
        info!("Set idle rate for {:?} to {:?}", id, dur);
    }

    fn get_idle_ms(&mut self, id: Option<ReportId>) -> Option<u32> {
        info!("Get idle rate for {:?}", id);
        None
    }
}

struct MyDeviceHandler {
    configured: AtomicBool,
}

impl MyDeviceHandler {
    fn new() -> Self {
        MyDeviceHandler {
            configured: AtomicBool::new(false),
        }
    }
}

impl Handler for MyDeviceHandler {
    fn enabled(&mut self, enabled: bool) {
        self.configured.store(false, Ordering::Relaxed);
        if enabled {
            info!("Device enabled");
        } else {
            info!("Device disabled");
        }
    }

    fn reset(&mut self) {
        self.configured.store(false, Ordering::Relaxed);
        info!("Bus reset, the Vbus current limit is 100mA");
    }

    fn addressed(&mut self, addr: u8) {
        self.configured.store(false, Ordering::Relaxed);
        info!("USB address set to: {}", addr);
    }

    fn configured(&mut self, configured: bool) {
        self.configured.store(configured, Ordering::Relaxed);
        if configured {
            info!(
                "Device configured, it may now draw up to the configured current limit from Vbus."
            )
        } else {
            info!("Device is no longer configured, the Vbus current limit is 100mA.");
        }
    }
}
