#![cfg_attr(not(feature = "std"), no_std)]

mod uld {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

    unsafe impl Send for VL53L7CX_Platform {}
    unsafe impl Send for VL53L7CX_Configuration {}
}

pub mod platform;

use core::{
    assert_eq,
    result::Result::{self, Err, Ok},
};
use platform::PlatformExt;

pub use uld::{
    VL53L7CX_Configuration as Configuration, VL53L7CX_ResultsData as ResultsData,
    VL53L7CX_API_REVISION, VL53L7CX_DEFAULT_I2C_ADDRESS,
};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum PowerMode {
    Sleep = uld::VL53L7CX_POWER_MODE_SLEEP,
    Wakeup = uld::VL53L7CX_POWER_MODE_WAKEUP,
}

impl PowerMode {
    fn from_u8(v: u8) -> Self {
        match v {
            uld::VL53L7CX_POWER_MODE_SLEEP => Self::Sleep,
            uld::VL53L7CX_POWER_MODE_WAKEUP => Self::Wakeup,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum RangingMode {
    Continuous = uld::VL53L7CX_RANGING_MODE_CONTINUOUS,
    Autonomous = uld::VL53L7CX_RANGING_MODE_AUTONOMOUS,
}

impl RangingMode {
    fn from_u8(v: u8) -> Self {
        match v {
            uld::VL53L7CX_RANGING_MODE_CONTINUOUS => Self::Continuous,
            uld::VL53L7CX_RANGING_MODE_AUTONOMOUS => Self::Autonomous,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Resolution {
    Res8x8 = uld::VL53L7CX_RESOLUTION_8X8,
    Res4x4 = uld::VL53L7CX_RESOLUTION_4X4,
}

impl Resolution {
    fn from_u8(v: u8) -> Self {
        match v {
            uld::VL53L7CX_RESOLUTION_8X8 => Self::Res8x8,
            uld::VL53L7CX_RESOLUTION_4X4 => Self::Res4x4,
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    StatusTimeout,
    StatusCorruptedFrame,
    StatusXtalkFailed,
    McuError,
    StatusInvalidParam,
    StatusError,
    Unknown,
}

impl Error {
    fn from_u8(e: u8) -> Self {
        match e {
            uld::VL53L7CX_STATUS_TIMEOUT_ERROR => Self::StatusTimeout,
            uld::VL53L7CX_STATUS_CORRUPTED_FRAME => Self::StatusCorruptedFrame,
            uld::VL53L7CX_STATUS_XTALK_FAILED => Self::StatusXtalkFailed,
            uld::VL53L7CX_MCU_ERROR => Self::McuError,
            uld::VL53L7CX_STATUS_INVALID_PARAM => Self::StatusInvalidParam,
            uld::VL53L7CX_STATUS_ERROR => Self::StatusError,
            _ => Self::Unknown,
        }
    }
}

fn wrap_result<T>(ret: u8, v: T) -> Result<T, Error> {
    match ret {
        uld::VL53L7CX_STATUS_OK => Ok(v),
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

        let pp = &raw mut config.platform.inner;

        unsafe {
            *(pp as *mut &mut dyn PlatformExt) = dy;
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

    pub fn ranging_frequency_hz(&mut self) -> Result<u8, Error> {
        unsafe {
            let mut hz = 0;
            wrap_result(
                uld::vl53l7cx_get_ranging_frequency_hz(self.as_ptr(), &raw mut hz),
                hz,
            )
        }
    }

    pub fn is_alive(&mut self) -> Result<bool, Error> {
        unsafe {
            let mut is_alive = 0;
            wrap_result(
                uld::vl53l7cx_is_alive(self.as_ptr(), &raw mut is_alive),
                is_alive == 1,
            )
        }
    }

    pub fn resolution(&mut self) -> Result<Resolution, Error> {
        unsafe {
            let mut resolution = 0;
            wrap_result(
                uld::vl53l7cx_get_resolution(self.as_ptr(), &raw mut resolution),
                Resolution::from_u8(resolution),
            )
        }
    }

    pub fn set_resolution(&mut self, resolution: Resolution) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_resolution(self.as_ptr(), resolution as u8),
                (),
            )
        }
    }

    pub fn power_mode(&mut self) -> Result<PowerMode, Error> {
        unsafe {
            let mut mode = 0;
            wrap_result(
                uld::vl53l7cx_get_power_mode(self.as_ptr(), &raw mut mode),
                PowerMode::from_u8(mode),
            )
        }
    }

    pub fn set_power_mode(&mut self, mode: PowerMode) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_set_power_mode(self.as_ptr(), mode as u8), ()) }
    }

    pub fn ranging_mode(&mut self) -> Result<RangingMode, Error> {
        unsafe {
            let mut mode = 0;
            wrap_result(
                uld::vl53l7cx_get_ranging_mode(self.as_ptr(), &raw mut mode),
                RangingMode::from_u8(mode),
            )
        }
    }

    pub fn set_ranging_mode(&mut self, mode: RangingMode) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_ranging_mode(self.as_ptr(), mode as u8),
                (),
            )
        }
    }

    pub fn integration_time_ms(&mut self) -> Result<u32, Error> {
        unsafe {
            let mut time_ms = 0;
            wrap_result(
                uld::vl53l7cx_get_integration_time_ms(self.as_ptr(), &raw mut time_ms),
                time_ms,
            )
        }
    }

    pub fn set_integration_time_ms(&mut self, time_ms: u32) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_integration_time_ms(self.as_ptr(), time_ms),
                (),
            )
        }
    }

    pub fn ranging_data(&mut self) -> Result<ResultsData, Error> {
        unsafe {
            let mut data = core::mem::MaybeUninit::<ResultsData>::uninit();
            wrap_result(
                uld::vl53l7cx_get_ranging_data(self.as_ptr(), data.as_mut_ptr()),
                data.assume_init(),
            )
        }
    }

    pub fn check_data_ready(&mut self) -> Result<bool, Error> {
        unsafe {
            let mut ready = 0;
            wrap_result(
                uld::vl53l7cx_check_data_ready(self.as_ptr(), &raw mut ready),
                ready == 1,
            )
        }
    }

    pub fn calibrate_xtalk(
        &mut self,
        reflectance_percent: u16,
        nb_samples: u8,
        distance_mm: u16,
    ) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_calibrate_xtalk(
                    self.as_ptr(),
                    reflectance_percent,
                    nb_samples,
                    distance_mm,
                ),
                (),
            )
        }
    }

    pub fn caldata_xtalk(
        &mut self,
    ) -> Result<[u8; uld::VL53L7CX_XTALK_BUFFER_SIZE as usize], Error> {
        let mut buffer = [0_u8; uld::VL53L7CX_XTALK_BUFFER_SIZE as usize];
        unsafe {
            wrap_result(
                uld::vl53l7cx_get_caldata_xtalk(self.as_ptr(), buffer.as_mut_ptr()),
                buffer,
            )
        }
    }

    pub fn set_caldata_xtalk(
        &mut self,
        cal_data_xtalk: &[u8; uld::VL53L7CX_XTALK_BUFFER_SIZE as usize],
    ) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_caldata_xtalk(self.as_ptr(), cal_data_xtalk.as_ptr() as *mut _),
                (),
            )
        }
    }

    pub fn sharpener_percent(&mut self) -> Result<u8, Error> {
        unsafe {
            let mut percent = 0;
            wrap_result(
                uld::vl53l7cx_get_sharpener_percent(self.as_ptr(), &raw mut percent),
                percent,
            )
        }
    }

    pub fn set_sharpener_percent(&mut self, percent: u8) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_sharpener_percent(self.as_ptr(), percent),
                (),
            )
        }
    }

    pub fn set_vhv_repeat_count(&mut self, repeat_count: u32) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_VHV_repeat_count(self.as_ptr(), repeat_count),
                (),
            )
        }
    }

    pub fn vhv_repeat_count(&mut self) -> Result<u32, Error> {
        unsafe {
            let mut repeat_count = 0;
            wrap_result(
                uld::vl53l7cx_get_VHV_repeat_count(self.as_ptr(), &raw mut repeat_count),
                repeat_count,
            )
        }
    }

    pub fn disable_internal_cp(&mut self) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_disable_internal_cp(self.as_ptr()), ()) }
    }

    pub fn enable_internal_cp(&mut self) -> Result<(), Error> {
        unsafe { wrap_result(uld::vl53l7cx_enable_internal_cp(self.as_ptr()), ()) }
    }

    pub fn set_i2c_address(&mut self, i2c_address: u16) -> Result<(), Error> {
        unsafe {
            wrap_result(
                uld::vl53l7cx_set_i2c_address(self.as_ptr(), i2c_address),
                (),
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
        fn rd_bytes(&mut self, _index: u16, _buf: &mut [u8]) -> Result<(), Error> {
            println!("rd_bytes");

            Ok(())
        }

        fn wr_bytes(&mut self, _index: u16, _vs: &[u8]) -> Result<(), Error> {
            println!("wr_bytes");

            Ok(())
        }

        fn delay_ms(&mut self, _ms: u32) -> Result<(), Error> {
            println!("delay_ms");

            Ok(())
        }

        fn on_i2c_address_changed(&mut self, _new_address: u8) {}
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
