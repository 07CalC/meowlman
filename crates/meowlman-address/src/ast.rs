#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddrSpec {
    pub local_part: LocalPart,
    pub domain: Domain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalPartPart {
    Atom(String),
    QuotedString(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalPart {
    pub parts: Vec<LocalPartPart>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    pub labels: Vec<String>,
}
