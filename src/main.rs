#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use defmt::*;
use core::mem;
use core::slice;
use embassy_executor::Spawner;
use embassy_futures::join::join;
use embassy_rp::bind_interrupts;
use embassy_rp::gpio::{Input, Pull};
use embassy_rp::peripherals::USB;
use embassy_rp::usb::Driver;
use embassy_usb::class::hid::{HidReaderWriter, ReportId, RequestHandler, State};
use embassy_usb::control::OutResponse;
use embassy_usb::{Builder, Handler};
use usbd_hid::descriptor::SerializedDescriptor;
use {defmt_rtt as _, panic_probe as _};

struct Range {
    pub min: f32,
    pub max: f32,
}

fn struct_to_bytes(report: &JoystickReport) -> &[u8] {
    unsafe {
        // Get a pointer to the struct and cast it to a pointer to u8
        let ptr = report as *const _ as *const u8;
        // Create a slice from the pointer with the size of the struct
        slice::from_raw_parts(ptr, mem::size_of::<JoystickReport>())
    }
}

fn convert_adc_to_hid_axis_value(adc_value: u16) -> i8 {
    let range = Range { min: 0.5, max: 2.8 };
    let voltage = (adc_value as f32 / 4095.0) * 3.3;
    let voltage = voltage - range.min;
    let mapped_value = -127.0 + ((voltage / (range.max - range.min)) * 254.0);
    mapped_value as i8
}

const JOYSTICK_HID_DESCRIPTOR: &[u8] = &[
    0x05, 0x01,        // Usage Page (Generic Desktop Ctrls)
    0x09, 0x04,        // Usage (Joystick)
    0xA1, 0x01,        // Collection (Application)
    
    // Throttle Control
    // Analog Controls 0
    0x09, 0x30,         // Usage (X Axis)
    0x15, 0x81,         // Logical Minimum (-127)
    0x25, 0x7F,         // Logical Maximum (127)
    0x35, 0x00,         // Physical Minimum (0)
    0x45, 0xFF,         // Physical Maximum (255)
    0x75, 0x08,         // Report Size (8 bits)
    0x95, 0x01,         // Report Count (1)
    0x81, 0x02,         // Input (Data, Var, Abs) - 1 byte axis

    // Analog Control 1
    0x09, 0x31,         // Usage (Y Axis)
    0x15, 0x81,         // Logical Minimum (-127)
    0x25, 0x7F,         // Logical Maximum (127)
    0x35, 0x00,         // Physical Minimum (0)
    0x45, 0xFF,         // Physical Maximum (255)
    0x75, 0x08,         // Report Size (8 bits)
    0x95, 0x01,         // Report Count (1)
    0x81, 0x02,         // Input (Data, Var, Abs) - 1 byte axis
    
    // Analog Control 2
    0x09, 0x32,         // Usage (Z Axis)
    0x15, 0x81,         // Logical Minimum (-127)
    0x25, 0x7F,         // Logical Maximum (127)
    0x35, 0x00,         // Physical Minimum (0)
    0x45, 0xFF,         // Physical Maximum (255)
    0x75, 0x08,         // Report Size (8 bits)
    0x95, 0x01,         // Report Count (1)
    0x81, 0x02,         // Input (Data, Var, Abs) - 1 byte axis
    
    // Buttons
    0x05, 0x09,        //   Usage Page (Button)
    0x19, 0x01,        //   Usage Minimum (Button 1)
    0x29, 0x17,        //   Usage Maximum (Button 23) (0x1E = 30 in hex)
    0x15, 0x00,        //   Logical Minimum (0)
    0x25, 0x01,        //   Logical Maximum (1)
    0x75, 0x01,        //   Report Size (1) - 1 bit per button
    0x95, 0x17,        //   Report Count (23) - 23 buttons
    0x81, 0x02,        //   Input (Data,Var,Abs)

    // Padding for unused bits (1 bits to make the byte boundary)
    0x75, 0x01,        //   Report Size (1) - Padding
    0x95, 0x01,        //   Report Count (1)
    0x81, 0x03,        //   Input (Const,Var,Abs) - Padding bits (not used)

    0xC0               // End Collection
];

// Define the report that will be sent over USB for the 30-button joystick (no axes)
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct JoystickReport {
    pub axis_x: i8,
    pub axis_y: i8,
    pub axis_z: i8,
    pub buttons: [u8; 3], // 32 buttons one u32 (32 bits total, but only 23 used)
}

impl Default for JoystickReport {
    fn default() -> Self {
        JoystickReport {
            axis_x: -127,
            axis_y: -127,
            axis_z: -127,
            buttons: [0, 0, 0],
        }
    }
}

impl SerializedDescriptor for JoystickReport {
    fn desc() -> &'static [u8] {
        JOYSTICK_HID_DESCRIPTOR
    }
}


bind_interrupts!(struct IrqsUsb {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<USB>;
});

bind_interrupts!(struct IrqsAdc {
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

struct Button {
    pub input: Input<'static>,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Initialise Peripherals
    let p = embassy_rp::init(Default::default());
    let adc_config = embassy_rp::adc::Config::default();
    
    let mut adc = embassy_rp::adc::Adc::new(p.ADC, IrqsAdc, adc_config);

    let driver = Driver::new(p.USB, IrqsUsb);
    let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
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

    let (reader, mut writer) = hid.split();

    let mut buttons_state = [
        Button { input: Input::new(p.PIN_0, Pull::Up), },
        Button { input: Input::new(p.PIN_1, Pull::Up), },
        Button { input: Input::new(p.PIN_2, Pull::Up), },
        Button { input: Input::new(p.PIN_3, Pull::Up), },
        Button { input: Input::new(p.PIN_4, Pull::Up), },
        Button { input: Input::new(p.PIN_5, Pull::Up), },
        Button { input: Input::new(p.PIN_6, Pull::Up), },
        Button { input: Input::new(p.PIN_7, Pull::Up), },
        Button { input: Input::new(p.PIN_8, Pull::Up), },
        Button { input: Input::new(p.PIN_9, Pull::Up), },
        Button { input: Input::new(p.PIN_10, Pull::Up), },
        Button { input: Input::new(p.PIN_11, Pull::Up), },
        Button { input: Input::new(p.PIN_12, Pull::Up), },
        Button { input: Input::new(p.PIN_13, Pull::Up), },
        Button { input: Input::new(p.PIN_14, Pull::Up), },
        Button { input: Input::new(p.PIN_15, Pull::Up), },
        Button { input: Input::new(p.PIN_16, Pull::Up), },
        Button { input: Input::new(p.PIN_17, Pull::Up), },
        Button { input: Input::new(p.PIN_18, Pull::Up), },
        Button { input: Input::new(p.PIN_19, Pull::Up), },
        Button { input: Input::new(p.PIN_20, Pull::Up), },
        Button { input: Input::new(p.PIN_21, Pull::Up), },
        Button { input: Input::new(p.PIN_22, Pull::Up), },
    ];

    let mut analog_p26 = embassy_rp::adc::Channel::new_pin(p.PIN_26, Pull::Down);
    let mut analog_p27 = embassy_rp::adc::Channel::new_pin(p.PIN_27, Pull::Down);
    let mut analog_p28 = embassy_rp::adc::Channel::new_pin(p.PIN_28, Pull::Down);

    for button in buttons_state.iter_mut() {
        button.input.set_schmitt(true);
    }

    let in_fut = async {
        loop {
            let p_26_adc_value = adc.read(&mut analog_p26).await.unwrap();
            let p_27_adc_value = adc.read(&mut analog_p27).await.unwrap();
            let p_28_adc_value = adc.read(&mut analog_p28).await.unwrap();
            let axis_x = convert_adc_to_hid_axis_value(p_26_adc_value); 
            let axis_y = convert_adc_to_hid_axis_value(p_27_adc_value); 
            let axis_z = convert_adc_to_hid_axis_value(p_28_adc_value); 

            let mut buttons: [u8; 3] = [0, 0, 0];

            for (index, button) in buttons_state.iter_mut().enumerate(){
                let byte_nr = index / 8;
                let bit_nr = index % 8;
                let pressed = button.input.is_low();
                if pressed {
                    buttons[byte_nr] += 0x1 << bit_nr;
                }
            }

            let report = JoystickReport { axis_x, axis_y, axis_z,
                 buttons,};

            match writer.write(struct_to_bytes(&report)).await {
                Ok(()) => {}
                Err(e) => warn!("Failed to send report: {:?}", e),
            };
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
