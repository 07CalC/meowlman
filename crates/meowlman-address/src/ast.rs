#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddrSpec {
    pub local_part: LocalPart,
    pub domain: Domain,
}

impl AddrSpec {
    pub fn address(&self) -> String {
        format!("{}@{}", self.local_part_string(), self.domain_string())
    }

    pub fn local_part_string(&self) -> String {
        self.local_part.to_string()
    }

    pub fn domain_string(&self) -> String {
        self.domain.to_string()
    }
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

impl LocalPart {
    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for (i, part) in self.parts.iter().enumerate() {
            if i > 0 {
                out.push('.');
            }
            match part {
                LocalPartPart::Atom(a) => out += a,
                LocalPartPart::QuotedString(qs) => {
                    out.push('"');
                    out += &qs;
                    out.push('"');
                }
            }
        }
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Domain {
    pub labels: Vec<String>,
}

impl Domain {
    pub fn to_string(&self) -> String {
        self.labels.join(".")
    }
}
