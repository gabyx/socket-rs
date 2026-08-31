use std::convert::From;

#[derive(Debug, Copy, Clone)]
pub enum Side {
    A,
    B,
}

impl From<&str> for Side {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "b" => Side::B,
            _ => Side::A,
        }
    }
}
