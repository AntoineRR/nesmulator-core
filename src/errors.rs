use std::{
    error::Error,
    fmt::{Debug, Display},
};

use crate::display_and_error_impl;

pub trait InvalidMemoryAccess: Debug {
    fn get_address(&self) -> u16;
    fn get_action(&self) -> String;
}

#[derive(Debug)]
pub struct InvalidMapperReadError(pub u16);

impl InvalidMemoryAccess for InvalidMapperReadError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("mapper read")
    }
}

display_and_error_impl!(InvalidMapperReadError);

#[derive(Debug)]
pub struct InvalidMapperWriteError(pub u16);

impl InvalidMemoryAccess for InvalidMapperWriteError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("mapper write")
    }
}

display_and_error_impl!(InvalidMapperWriteError);

#[derive(Debug)]
pub struct InvalidPPUBusReadError(pub u16);

impl InvalidMemoryAccess for InvalidPPUBusReadError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("ppu bus read")
    }
}

display_and_error_impl!(InvalidPPUBusReadError);

#[derive(Debug)]
pub struct InvalidPPUBusWriteError(pub u16);

impl InvalidMemoryAccess for InvalidPPUBusWriteError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("ppu bus write")
    }
}

display_and_error_impl!(InvalidPPUBusWriteError);

#[derive(Debug)]
pub struct InvalidPPURegisterReadError(pub u16);

impl InvalidMemoryAccess for InvalidPPURegisterReadError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("ppu register read")
    }
}

display_and_error_impl!(InvalidPPURegisterReadError);

#[derive(Debug)]
pub struct InvalidPPURegisterWriteError(pub u16);

impl InvalidMemoryAccess for InvalidPPURegisterWriteError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("ppu register write")
    }
}

display_and_error_impl!(InvalidPPURegisterWriteError);

#[derive(Debug)]
pub struct InvalidAPURegisterReadError(pub u16);

impl InvalidMemoryAccess for InvalidAPURegisterReadError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("apu register read")
    }
}

display_and_error_impl!(InvalidAPURegisterReadError);

#[derive(Debug)]
pub struct InvalidAPURegisterWriteError(pub u16);

impl InvalidMemoryAccess for InvalidAPURegisterWriteError {
    fn get_address(&self) -> u16 {
        self.0
    }

    fn get_action(&self) -> String {
        String::from("apu register write")
    }
}

display_and_error_impl!(InvalidAPURegisterWriteError);

#[macro_export]
macro_rules! display_and_error_impl {
    ($t: ty) => {
        impl Display for $t {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    f,
                    "Invalid {} at {:#X}",
                    self.get_action(),
                    self.get_address()
                )
            }
        }

        impl Error for $t {}
    };
}
