#![allow(non_snake_case)]

use crate::uld::VL53L7CX_Platform;
use crate::uld::VL53L7CX_STATUS_OK;
use core::ops::Fn;
use core::ptr::addr_of_mut;

pub trait PlatformExt {
    fn rd_bytes(&mut self, index: u16, buf: &mut [u8]);
    fn wr_bytes(&mut self, index: u16, vs: &[u8]);
    fn delay_ms(&mut self, ms: u32);
}

#[no_mangle]
extern "C" fn VL53L7CX_RdByte(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *mut u8,
) -> u8 {
    with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts_mut(p_value, 1) };
        p.rd_bytes(RegisterAdress, buf);
    });

    VL53L7CX_STATUS_OK
}

#[no_mangle]
extern "C" fn VL53L7CX_WrByte(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    value: u8,
) -> u8 {
    with_inner(p_platform, |p| {
        p.wr_bytes(RegisterAdress, &[value]);
    });

    VL53L7CX_STATUS_OK
}

#[no_mangle]
extern "C" fn VL53L7CX_RdMulti(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *mut u8,
    size: u32,
) -> u8 {
    with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts_mut(p_value, size as usize) };
        p.rd_bytes(RegisterAdress, buf);
    });

    VL53L7CX_STATUS_OK
}

#[no_mangle]
extern "C" fn VL53L7CX_WrMulti(
    p_platform: *mut VL53L7CX_Platform,
    RegisterAdress: u16,
    p_value: *const u8,
    size: u32,
) -> u8 {
    with_inner(p_platform, |p| {
        let buf = unsafe { core::slice::from_raw_parts(p_value, size as usize) };
        p.wr_bytes(RegisterAdress, buf);
    });

    VL53L7CX_STATUS_OK
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
    with_inner(p_platform, |p| {
        p.delay_ms(TimeMs);
    });
    VL53L7CX_STATUS_OK
}

fn with_inner(p_platform: *mut VL53L7CX_Platform, f: impl Fn(&mut dyn PlatformExt)) {
    let inner: &mut dyn PlatformExt = unsafe {
        let mut platform = p_platform.read();
        let pp = addr_of_mut!(platform.inner);
        // println!("ptr {:?}", pp);
        // println!("ptr i2c {:?}", p_platform.read().address);
        core::ptr::NonNull::new_unchecked(pp)
            .cast::<&mut dyn PlatformExt>()
            .read()
    };

    f(inner)
}
