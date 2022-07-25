use std::fmt;

#[derive(Debug)]
pub enum Currency {
    CAD,
    USD,
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TryFrom<&str> for Currency {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "CAD" => Ok(Self::CAD),
            "USD" => Ok(Self::USD),
            _ => Err(anyhow::anyhow!("unknown currency {}", value)),
        }
    }
}
