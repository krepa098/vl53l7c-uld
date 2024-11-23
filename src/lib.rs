use core::ptr::addr_of_mut;
use platform::PlatformExt;

mod uld {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub use uld::VL53L7CX_Configuration as Configuration;

pub mod platform;

pub const VL53L7CX_STATUS_OK: u8 = 0;
pub const VL53L7CX_STATUS_TIMEOUT_ERROR: u8 = 1;
pub const VL53L7CX_STATUS_CORRUPTED_FRAME: u8 = 2;
pub const VL53L7CX_STATUS_XTALK_FAILED: u8 = 4;
pub const VL53L7CX_MCU_ERROR: u8 = 66;
pub const VL53L7CX_STATUS_INVALID_PARAM: u8 = 127;
pub const VL53L7CX_STATUS_ERROR: u8 = 255;

// the default (shifted) i2c address
pub const VL53L7CX_DEFAULT_I2C_ADDRESS: u8 = 0x52;

#[derive(Debug, Clone, Copy)]
pub enum Error {
    StatusTimeout,
    StatusCorruptedFrame,
    StatusXtalkFailed,
    McuError,
    StatusInvalidParam,
    StatusError,
}

impl Error {
    fn from_u8(e: u8) -> Self {
        match e {
            VL53L7CX_STATUS_TIMEOUT_ERROR => Self::StatusTimeout,
            VL53L7CX_STATUS_CORRUPTED_FRAME => Self::StatusCorruptedFrame,
            VL53L7CX_STATUS_XTALK_FAILED => Self::StatusXtalkFailed,
            VL53L7CX_MCU_ERROR => Self::McuError,
            VL53L7CX_STATUS_INVALID_PARAM => Self::StatusInvalidParam,
            VL53L7CX_STATUS_ERROR => Self::StatusError,
            _ => unimplemented!(),
        }
    }
}

fn wrap_result<T>(ret: u8, v: T) -> Result<T, Error> {
    match ret {
        VL53L7CX_STATUS_OK => Ok(v),
        _ => Err(Error::from_u8(ret)),
    }
}

impl uld::VL53L7CX_Configuration {
    pub fn new<P: PlatformExt + 'static>(p: &mut P) -> uld::VL53L7CX_Configuration {
        let mut config: uld::VL53L7CX_Configuration = unsafe { core::mem::zeroed() };

        // note: this is a *fat* pointer (size of two pointers)
        let dy: &mut dyn PlatformExt = p;

        assert_eq!(
            size_of::<&mut dyn PlatformExt>(),
            size_of_val(&config.platform.inner)
        );

        let pp = addr_of_mut!(config.platform.inner);

        unsafe {
            *(pp as *mut &mut dyn PlatformExt) = dy;
            config.platform.address = VL53L7CX_DEFAULT_I2C_ADDRESS as u16;
            println!("config.platform {:?}", config.platform);
        }
        config
    }

    #[inline]
    pub fn as_ptr(&mut self) -> *mut Self {
        self as *mut _
    }

    pub fn init(&mut self) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_init(self.as_ptr()), ()) }
    }

    pub fn start_ranging(&mut self) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_start_ranging(self.as_ptr()), ()) }
    }

    pub fn stop_ranging(&mut self) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_stop_ranging(self.as_ptr()), ()) }
    }

    pub fn set_ranging_frequency_hz(&mut self, hz: u8) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_ranging_frequency_hz(self.as_ptr(), hz),
                (),
            )
        }
    }

    pub fn get_ranging_frequency_hz(&mut self) -> Result<u8, Error> {
        unsafe {
            let mut hz = 0;
            wrap_result(
                uld::vl53l7cx_get_ranging_frequency_hz(self.as_ptr(), &mut hz as *mut _),
                hz,
            )
        }
    }

    pub fn is_alive(&mut self) -> Result<u8, Error> {
        unsafe {
            let mut is_alive = 0;
            wrap_result(
                uld::vl53l7cx_is_alive(self.as_ptr(), &mut is_alive as *mut _),
                is_alive,
            )
        }
    }

    pub fn get_resolution(&mut self) -> Result<u8, Error> {
        unsafe {
            let mut resolution = 0;
            wrap_result(
                uld::vl53l7cx_get_resolution(self.as_ptr(), &mut resolution as *mut _),
                resolution,
            )
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[derive(Default)]
    pub struct DummyPlatform {
        _pin: core::marker::PhantomPinned,
    }

    impl PlatformExt for DummyPlatform {
        fn rd_bytes(&mut self, _index: u16, _buf: &mut [u8]) {
            println!("rd_bytes");
        }

        fn wr_bytes(&mut self, _index: u16, _vs: &[u8]) {
            println!("wr_bytes");
        }

        fn delay_ms(&mut self, _ms: u32) {
            println!("delay_ms");
        }
    }

    #[test]
    fn init() {
        let mut platform = DummyPlatform::default();

        let mut dev = uld::VL53L7CX_Configuration::new(&mut platform);

        println!("Dev size {}kb", size_of_val(&dev) / 1024);

        // this timeouts since we don't actually interact with the device
        assert!(dev.init().is_err())
    }
}
