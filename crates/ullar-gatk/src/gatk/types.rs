use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GatkSortOrder {
    Coordinate,
    Queryname,
    Duplicate,
    Unsorted,
}

impl FromStr for GatkSortOrder {
    type Err = ();

    fn from_str(input: &str) -> Result<GatkSortOrder, Self::Err> {
        match input.to_lowercase().as_str() {
            "coordinate" => Ok(GatkSortOrder::Coordinate),
            "queryname" => Ok(GatkSortOrder::Queryname),
            "duplicate" => Ok(GatkSortOrder::Duplicate),
            "unsorted" => Ok(GatkSortOrder::Unsorted),
            _ => Err(()),
        }
    }
}

impl ToString for GatkSortOrder {
    fn to_string(&self) -> String {
        match self {
            GatkSortOrder::Coordinate => "coordinate".to_string(),
            GatkSortOrder::Queryname => "queryname".to_string(),
            GatkSortOrder::Duplicate => "duplicate".to_string(),
            GatkSortOrder::Unsorted => "unsorted".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum GatkFormat {
    Bam,
    Sam,
    Cram,
}

impl FromStr for GatkFormat {
    type Err = ();

    fn from_str(input: &str) -> Result<GatkFormat, Self::Err> {
        match input.to_lowercase().as_str() {
            "bam" => Ok(GatkFormat::Bam),
            "sam" => Ok(GatkFormat::Sam),
            "cram" => Ok(GatkFormat::Cram),
            _ => Err(()),
        }
    }
}

impl ToString for GatkFormat {
    fn to_string(&self) -> String {
        match self {
            GatkFormat::Bam => "BAM".to_string(),
            GatkFormat::Sam => "SAM".to_string(),
            GatkFormat::Cram => "CRAM".to_string(),
        }
    }
}
