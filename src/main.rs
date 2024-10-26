#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::adc::Adc;
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
    config.manufacturer = Some("Bor(TM)");
    config.product = Some("Simwheel");
    config.serial_number = Some("0xCAFEBABE");
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

    let mut gpio0 = Input::new(p.PIN_0, Pull::Up);
    let mut gpio1 = Input::new(p.PIN_1, Pull::Up);
    let mut gpio2 = Input::new(p.PIN_2, Pull::Up);
    let mut gpio3 = Input::new(p.PIN_3, Pull::Up);
    let mut gpio4 = Input::new(p.PIN_4, Pull::Up);
    let mut gpio5 = Input::new(p.PIN_5, Pull::Up);
    let mut gpio6 = Input::new(p.PIN_6, Pull::Up);
    let mut gpio7 = Input::new(p.PIN_7, Pull::Up);
    let mut gpio8 = Input::new(p.PIN_8, Pull::Up);
    let mut gpio9 = Input::new(p.PIN_9, Pull::Up);
    let mut gpio10 = Input::new(p.PIN_10, Pull::Up);
    let mut gpio11 = Input::new(p.PIN_11, Pull::Up);
    let mut gpio12 = Input::new(p.PIN_12, Pull::Up);
    let mut gpio13 = Input::new(p.PIN_13, Pull::Up);
    let mut gpio14 = Input::new(p.PIN_14, Pull::Up);
    let mut gpio15 = Input::new(p.PIN_15, Pull::Up);
    let mut gpio16 = Input::new(p.PIN_16, Pull::Up);
    let mut gpio17 = Input::new(p.PIN_17, Pull::Up);
    let mut gpio18 = Input::new(p.PIN_18, Pull::Up);
    let mut gpio19 = Input::new(p.PIN_19, Pull::Up);
    let mut gpio20 = Input::new(p.PIN_20, Pull::Up);
    let mut gpio21 = Input::new(p.PIN_21, Pull::Up);
    let mut gpio22 = Input::new(p.PIN_22, Pull::Up);
    let mut gpio23 = Input::new(p.PIN_23, Pull::Up);
    let mut gpio24 = Input::new(p.PIN_24, Pull::Up);
    let mut gpio25 = Input::new(p.PIN_25, Pull::Up);
    let mut gpio26 = Input::new(p.PIN_26, Pull::Up);
    let mut gpio27 = Input::new(p.PIN_27, Pull::Up);
    let mut gpio28 = Input::new(p.PIN_28, Pull::Up);
    let mut gpio29 = Input::new(p.PIN_29, Pull::Up);

    gpio0.set_schmitt(true);
    gpio1.set_schmitt(true);
    gpio2.set_schmitt(true);
    gpio3.set_schmitt(true);
    gpio4.set_schmitt(true);
    gpio5.set_schmitt(true);
    gpio6.set_schmitt(true);
    gpio7.set_schmitt(true);
    gpio8.set_schmitt(true);
    gpio9.set_schmitt(true);
    gpio10.set_schmitt(true);
    gpio11.set_schmitt(true);
    gpio12.set_schmitt(true);
    gpio13.set_schmitt(true);
    gpio14.set_schmitt(true);
    gpio15.set_schmitt(true);
    gpio16.set_schmitt(true);
    gpio17.set_schmitt(true);
    gpio18.set_schmitt(true);
    gpio19.set_schmitt(true);
    gpio20.set_schmitt(true);
    gpio21.set_schmitt(true);
    gpio22.set_schmitt(true);
    gpio23.set_schmitt(true);
    gpio24.set_schmitt(true);
    gpio25.set_schmitt(true);
    gpio26.set_schmitt(true);
    gpio27.set_schmitt(true);
    gpio28.set_schmitt(true);
    gpio29.set_schmitt(true);
    
    let (reader, mut writer) = hid.split();

    let now = Instant::now();
    let mut gpio_info = [
        GpioInfo { buttons: [0b0000_0001, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0010, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0100, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_1000, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0001_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0010_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0100_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b1000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0001, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0010, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0100, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_1000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0001_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0010_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0100_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b1000_0000, 0b0000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0001, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0010, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0100, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_1000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0001_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0010_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0100_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b1000_0000, 0b0000_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_0100], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0000_1000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0001_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0010_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b0100_0000], last_time: now, previous_pressed: false },
        GpioInfo { buttons: [0b0000_0000, 0b0000_0000, 0b0000_0000, 0b1000_0000], last_time: now, previous_pressed: false },
    ];

    // Do stuff with the class!
    let in_fut = async {
        loop {
            let (_, index) = embassy_futures::select::select_array([
                gpio0.wait_for_any_edge(),
                gpio1.wait_for_any_edge(),
                gpio2.wait_for_any_edge(),
                gpio3.wait_for_any_edge(),
                gpio4.wait_for_any_edge(),
                gpio5.wait_for_any_edge(),
                gpio6.wait_for_any_edge(),
                gpio7.wait_for_any_edge(),
                gpio8.wait_for_any_edge(),
                gpio9.wait_for_any_edge(),
                gpio10.wait_for_any_edge(),
                gpio11.wait_for_any_edge(),
                gpio12.wait_for_any_edge(),
                gpio13.wait_for_any_edge(),
                gpio14.wait_for_any_edge(),
                gpio15.wait_for_any_edge(),
                gpio16.wait_for_any_edge(),
                gpio17.wait_for_any_edge(),
                gpio18.wait_for_any_edge(),
                gpio19.wait_for_any_edge(),
                gpio20.wait_for_any_edge(),
                gpio21.wait_for_any_edge(),
                gpio22.wait_for_any_edge(),
                gpio23.wait_for_any_edge(),
                gpio24.wait_for_any_edge(),
                gpio25.wait_for_any_edge(),
                gpio26.wait_for_any_edge(),
                gpio27.wait_for_any_edge(),
                gpio28.wait_for_any_edge(),
                gpio29.wait_for_any_edge(),
            ]).await;

            let pressed = match index {
                0 => gpio0.is_low(),
                1 => gpio1.is_low(),
                2 => gpio2.is_low(),
                3 => gpio3.is_low(),
                4 => gpio4.is_low(),
                5 => gpio5.is_low(),
                6 => gpio6.is_low(),
                7 => gpio7.is_low(),
                8 => gpio8.is_low(),
                9 => gpio9.is_low(),
                10 => gpio10.is_low(),
                11 => gpio11.is_low(),
                12 => gpio12.is_low(),
                13 => gpio13.is_low(),
                14 => gpio14.is_low(),
                15 => gpio15.is_low(),
                16 => gpio16.is_low(),
                17 => gpio17.is_low(),
                18 => gpio18.is_low(),
                19 => gpio19.is_low(),
                20 => gpio20.is_low(),
                21 => gpio21.is_low(),
                22 => gpio22.is_low(),
                23 => gpio23.is_low(),
                24 => gpio24.is_low(),
                25 => gpio25.is_low(),
                26 => gpio26.is_low(),
                27 => gpio27.is_low(),
                28 => gpio28.is_low(),
                29 => gpio29.is_low(),
                _ => false,
            };
            gpio_info[index].last_time = Instant::now();

//            gpio28

            if gpio_info[index].previous_pressed != pressed {
                let report = if pressed {
                    JoystickReport { buttons: gpio_info[index].buttons }
                } else {
                    JoystickReport { buttons: [0, 0, 0, 0] }
                };
                match writer.write(&report.buttons).await {
                    Ok(()) => {}
                    Err(e) => warn!("Failed to send report: {:?}", e),
                };
            }
            gpio_info[index].previous_pressed = pressed;
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
