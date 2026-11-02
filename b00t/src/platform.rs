#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Soc {
    T8103,
    T8112,
    T8122,
    T8132,
    T8140,
    T8142,
    T8152,
    Unknown,
}

impl Soc {
    pub fn from_compatible(compatible: &str) -> Self {
        if compatible.contains("t8103") {
            Soc::T8103
        } else if compatible.contains("t8112") {
            Soc::T8112
        } else if compatible.contains("t8122") {
            Soc::T8122
        } else if compatible.contains("t8132") {
            Soc::T8132
        } else if compatible.contains("t8142") {
            Soc::T8142
        } else if compatible.contains("t8140") {
            Soc::T8140
        } else if compatible.contains("t8152") {
            Soc::T8152
        } else {
            Soc::Unknown
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Soc::T8103 => "t8103 (M1)",
            Soc::T8112 => "t8112 (M2)",
            Soc::T8122 => "t8122 (M3)",
            Soc::T8132 => "t8132 (M3 Pro/Max)",
            Soc::T8140 => "t8140 (M4)",
            Soc::T8142 => "t8142 (M4 Pro/Max)",
            Soc::T8152 => "t8152 (M5?)",
            Soc::Unknown => "unknown",
        }
    }

    pub fn aic_compatible(self) -> &'static str {
        match self {
            Soc::T8103 | Soc::T8112 => "apple,t8103-aic",
            Soc::T8122 | Soc::T8132 => "apple,t8122-aic3",
            Soc::T8140 | Soc::T8142 | Soc::T8152 => "apple,t8140-aic3",
            Soc::Unknown => "apple,aic",
        }
    }

    pub fn primary_target(self) -> bool {
        matches!(self, Soc::T8122)
    }
}

pub const SUPPORTED_SOCS: &[Soc] = &[Soc::T8122];

pub fn is_supported(soc: Soc) -> bool {
    SUPPORTED_SOCS.contains(&soc)
}
