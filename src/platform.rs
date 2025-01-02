#![allow(non_snake_case)]

use crate::{
    uld::{VL53L7CX_Platform, VL53L7CX_STATUS_OK},
    Error,
};
use core::ops::Fn;

pub trait PlatformExt {
    fn rd_bytes(&mut self, register: u16, buf: &mut [u8]) -> Result<(), Error>;
    fn wr_bytes(&mut self, register: u16, buf: &[u8]) -> Result<(), Error>;
    fn delay_ms(&mut self, ms: u32) -> Result<(), Error>;
    fn on_i2c_address_changed(&mut self, new_address: u8);
}

#[no_mangle]
extern "C" fn VL53L7CX_RdByte(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *mut u8,
) -> u8 {
    let r = with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts_mut(p_value, 1) };
        p.rd_bytes(RegisterAdress, buf)
    });

    match r {
        Ok(_) => VL53L7CX_STATUS_OK,
        Err(e) => e as u8,
    }
}

#[no_mangle]
extern "C" fn VL53L7CX_WrByte(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    value: u8,
) -> u8 {
    let r = with_inner(p_platform, |p| p.wr_bytes(RegisterAdress, &[value]));

    match r {
        Ok(_) => VL53L7CX_STATUS_OK,
        Err(e) => e as u8,
    }
}

#[no_mangle]
extern "C" fn VL53L7CX_RdMulti(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *mut u8,
    size: u32,
) -> u8 {
    let r = with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts_mut(p_value, size as usize) };
        p.rd_bytes(RegisterAdress, buf)
    });

    match r {
        Ok(_) => VL53L7CX_STATUS_OK,
        Err(e) => e as u8,
    }
}

#[no_mangle]
extern "C" fn VL53L7CX_WrMulti(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *const u8,
    size: u32,
) -> u8 {
    let r = with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts(p_value, size as usize) };
        p.wr_bytes(RegisterAdress, buf)
    });

    match r {
        Ok(_) => VL53L7CX_STATUS_OK,
        Err(e) => e as u8,
    }
}

#[no_mangle]
extern "C" fn VL53L7CX_Reset_Sensor(_p_platform: *mut VL53L7CX_Platform) -> u8 {
    VL53L7CX_STATUS_OK
}

#[no_mangle]
extern "C" fn VL53L7CX_SwapBuffer(buffer: *mut u8, size: u32) -> u8 {
    // SwapBuffer: ABCD becomes DCBA
    let buffer = unsafe { core::slice::from_raw_parts_mut(buffer, size as usize) };

    for chunk in buffer.chunks_exact_mut(4) {
        chunk.swap(0, 3);
        chunk.swap(1, 2);
    }

    VL53L7CX_STATUS_OK
}

#[no_mangle]
extern "C" fn VL53L7CX_WaitMs(p_platform: *mut VL53L7CX_Platform, TimeMs: u32) -> u8 {
    let r = with_inner(p_platform, |p| p.delay_ms(TimeMs));

    match r {
        Ok(_) => VL53L7CX_STATUS_OK,
        Err(e) => e as u8,
    }
}

#[no_mangle]
extern "C" fn VL53L7CX_I2CAddressChanged(p_platform: *mut VL53L7CX_Platform, address: u16) {
    with_inner(p_platform, |p| p.on_i2c_address_changed(address as u8));
}

fn with_inner<T, F>(p_platform: *mut VL53L7CX_Platform, f: F) -> T
where
    F: Fn(&mut dyn PlatformExt) -> T,
{
    let inner: &mut dyn PlatformExt = unsafe {
        let mut platform = p_platform.read();
        let pp = &raw mut platform.inner;
        core::ptr::NonNull::new_unchecked(pp)
            .cast::<&mut dyn PlatformExt>()
            .read()
    };

    f(inner)
}
