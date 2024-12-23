# VL53L7C ULD Rust Wrapper
This is a wrapper around ST's `VL53L7C` ULD driver (version 2.0.0) to be used with Rust.

## Usage
The ULD driver is compiled as a static library and then linked to your project. The driver needs to know how to interact with the hardware (i2c read/write and delay). This functionality is defined by the `PlatformExt` trait that you will have to implement for your hardware.

Take the following snipped as a starting point:

```
struct Inner {
    bus: I2c,
    configuration: Configuration,
}

pub struct Platform {
    inner: Option<Inner>,
}

impl Platform {
    pub const fn new() -> Self {
        Self { inner: None }
    }

    pub fn init(
        &mut self,
        bus: I2c,
    ) {
        self.inner = Some(Inner {
            bus,
            configuration: Configuration::new(self),
        });
    }

    pub fn with<F, T>(&mut self, f: F) -> T
    where
        F: Fn(DeviceChannel, &mut Configuration) -> T,
    {
        // (you may extend this function to support multiple
        // devices on the same bus using the lpn pin)
        // !! make sure lpn is high to select the device
        let r = f(ch, &mut self.inner().configuration);
        // !! make sure lpn is low to deselect the device
        r
    }

    fn inner(&mut self) -> &mut Inner {
        defmt::unwrap!(self.inner.as_mut())
    }
}

impl PlatformExt for Platform {
    fn rd_bytes(&mut self, register: u16, buf: &mut [u8]) -> Result<(), Error> {
        self.inner()
            .bus
            .blocking_write_read(
                vl53l7c_uld::VL53L7CX_DEFAULT_I2C_ADDRESS >> 1,
                &register.to_be_bytes(),
                buf,
            )
            .map_err(|_| vl53l7c_uld::Error::StatusError)
    }

    fn wr_bytes(&mut self, register: u16, buf: &[u8]) -> Result<(), Error> {
        self.inner()
            .bus
            .blocking_write_vectored(
                vl53l7c_uld::VL53L7CX_DEFAULT_I2C_ADDRESS >> 1,
                &[&register.to_be_bytes(), buf],
            )
            .map_err(|_| vl53l7c_uld::Error::StatusError)
    }

    fn delay_ms(&mut self, ms: u32) -> Result<(), Error> {
        embassy_futures::block_on(crate::Mono::delay(MicrosDurationU64::millis(ms as u64)));
        Ok(())
    }
}
```

Then create and init the platform and start ranging:
```
// setup the platform
let mut platform = Platform::new();
platform.init(i2c);

// start ranging
platform.with(|conf| conf.start_ranging().unwrap());

// get the results
loop {
    let data = platform.with(|conf| conf.ranging_data().unwrap());
}
```

## Acknowledgements
This work is partially based on the fat-pointer idea of the [ZOO-esp32](https://github.com/lure23/ZOO-esp32) project.

## License
BSD-3-Clause (https://opensource.org/licenses/BSD-3-Clause)