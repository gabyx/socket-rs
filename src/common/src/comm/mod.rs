use clap::ValueEnum;
use std::convert::From;

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum Side {
    A,
    B,
}

impl Side {
    #[must_use]
    pub fn port(self) -> u16 {
        match self {
            Side::A => 10010,
            Side::B => 10011,
        }
    }
}

impl<T: AsRef<str>> From<T> for Side {
    fn from(value: T) -> Self {
        match value.as_ref().to_lowercase().as_str() {
            "b" => Side::B,
            _ => Side::A,
        }
    }
}

impl From<Side> for &'static str {
    fn from(side: Side) -> Self {
        match side {
            Side::A => "a",
            Side::B => "b",
        }
    }
}
