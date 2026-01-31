use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SamtoolsIndexFormat {
    Fai,
    FaiGz,
}

impl SamtoolsIndexFormat {
    pub fn extension(&self) -> &str {
        match self {
            SamtoolsIndexFormat::Fai => "fai",
            SamtoolsIndexFormat::FaiGz => "fai.gz",
        }
    }
}

impl FromStr for SamtoolsIndexFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fai" => Ok(SamtoolsIndexFormat::Fai),
            "fai.gz" => Ok(SamtoolsIndexFormat::FaiGz),
            _ => Err(format!("Unknown SamtoolsIndexFormat: {}", s)),
        }
    }
}

impl ToString for SamtoolsIndexFormat {
    fn to_string(&self) -> String {
        match self {
            SamtoolsIndexFormat::Fai => "fai".to_string(),
            SamtoolsIndexFormat::FaiGz => "fai.gz".to_string(),
        }
    }
}
