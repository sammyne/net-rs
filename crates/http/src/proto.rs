use std::convert::From;
use std::fmt::Display;
use std::str::FromStr;

pub struct Proto {
    pub major: u32,
    pub minor: u32,
}

impl Default for Proto {
    fn default() -> Self {
        Self { major: 2, minor: 0 }
    }
}

impl Display for Proto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP/{}.{}", self.major, self.minor)
    }
}

impl From<hyper::Version> for Proto {
    fn from(v: hyper::Version) -> Self {
        use hyper::Version;

        let (major, minor) = match v {
            Version::HTTP_09 => (0, 9),
            Version::HTTP_10 => (1, 0),
            Version::HTTP_11 => (1, 1),
            Version::HTTP_2 => (2, 0),
            Version::HTTP_3 => (3, 0),
            _ => panic!("unsupported hyper version"),
        };

        Self { major, minor }
    }
}

impl FromStr for Proto {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = s
            .strip_prefix("HTTP/")
            .ok_or_else(|| "missing prefix 'HTTP/'".to_string())?;

        let (major_str, minor_str) = v
            .split_once('.')
            .ok_or_else(|| "miss '.' separating major and minor".to_string())?;

        let major = major_str
            .parse::<u32>()
            .map_err(|err| format!("parse major: {}", err))?;

        let minor = minor_str
            .parse::<u32>()
            .map_err(|err| format!("parse minor: {}", err))?;

        Ok(Self { major, minor })
    }
}
