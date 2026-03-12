use defmt::error;
use stm32f4xx_hal::{i2c, prelude::*, rcc::Rcc};

pub struct Imu<I: i2c::Instance> {
  bus: i2c::I2c<I>,
}

mod addr {
  pub const CONFIG: u8 = 0x1a;
  pub const GYRO_CONFIG: u8 = 0x1b;
  pub const ACCEL_CONFIG: u8 = 0x1c;
  pub const ACCEL_XOUT_H: u8 = 0x3b;
  pub const GYRO_XOUT_H: u8 = 0x43;
  pub const PWR_MGMT_1: u8 = 0x6b;
}

impl<I: i2c::Instance> Imu<I> {
  pub fn new(
    i2c: I,
    scl: impl Into<I::Scl>,
    sda: impl Into<I::Sda>,
    rcc: &mut Rcc,
  ) -> Result<Self, ()> {
    let mut imu = Imu { bus: i2c.i2c((scl, sda), 400.kHz(), rcc) };

    imu.write(addr::CONFIG, 0x00)?; // Input disabled and highest possible bandwidth
    imu.write(addr::GYRO_CONFIG, 0x08)?; // 500 deg/sec, no self test
    imu.write(addr::ACCEL_CONFIG, 0x00)?; // 2g, no self test
    imu.write(addr::PWR_MGMT_1, 0x01)?; // Clock from the X axis of the gyro

    Ok(imu)
  }

  pub fn read_accel(&mut self) -> [i16; 3] {
    let [xh, xl, yh, yl, zh, zl] = self.read::<6>(addr::ACCEL_XOUT_H).unwrap();
    [[xh, xl], [yh, yl], [zh, zl]].map(i16::from_be_bytes)
  }

  pub fn read_gyro(&mut self) -> [i16; 3] {
    let [xh, xl, yh, yl, zh, zl] = self.read::<6>(addr::GYRO_XOUT_H).unwrap();
    [[xh, xl], [yh, yl], [zh, zl]].map(i16::from_be_bytes)
  }

  pub fn read<const N: usize>(&mut self, addr: u8) -> Result<[u8; N], ()> {
    let mut buf = [0_u8; N];
    match self.bus.write_read(i2c::Address::Seven(0x68), &[addr], &mut buf) {
      Ok(()) => Ok(buf),
      Err(_) => {
        error!("error reading from IMU");
        Err(())
      }
    }
  }

  pub fn write(&mut self, addr: u8, value: u8) -> Result<(), ()> {
    match self.bus.write(i2c::Address::Seven(0x68), &[addr, value]) {
      Ok(()) => Ok(()),
      Err(_) => {
        error!("error writing to IMU");
        Err(())
      }
    }
  }
}
