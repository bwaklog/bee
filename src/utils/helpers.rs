// NOTE: SEP 30, 2024
// commit pending tests

use core::{fmt, net};
use std::net::{SocketAddr, SocketAddrV4};
use std::{fs, path::PathBuf, str::FromStr};
// FIX: Handle error type for yaml
use yaml_rust2::{Yaml, YamlLoader};

pub type HelperErrorResult<T> = Result<T, HelperErrors>;

#[derive(Debug)]
pub enum HelperErrors {
    IOError(std::io::Error),
    SocketParserError(net::AddrParseError),
    ParserYamlError,
}

impl fmt::Display for HelperErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HelperErrors::ParserYamlError => write!(
                f,
                "Custom yaml_rust2::Yaml parser error {}:{}",
                file!(),
                line!()
            ),
            HelperErrors::SocketParserError(err) => write!(f, "Socket Parse Error: {}", err),
            HelperErrors::IOError(err) => write!(f, "IOError: {}", err),
        }
    }
}

impl From<std::net::AddrParseError> for HelperErrors {
    fn from(value: std::net::AddrParseError) -> Self {
        HelperErrors::SocketParserError(value)
    }
}

impl From<std::io::Error> for HelperErrors {
    fn from(value: std::io::Error) -> Self {
        HelperErrors::IOError(value)
    }
}

#[derive(Debug)]
pub struct RaftConfig {
    listener_addr: SocketAddr,
    connections: Vec<SocketAddr>,
}

#[derive(Debug)]
pub struct StoreConfig {
    local_path: PathBuf,
}

#[derive(Debug)]
pub struct Config {
    pub node_name: String,
    pub raft: RaftConfig,
    pub store: StoreConfig,
}

pub fn parse_config() -> HelperErrorResult<Config> {
    let content = fs::read_to_string("config.yml")?;
    let yaml_content = YamlLoader::load_from_str(&*content).unwrap();

    let metadata = from_yaml_hash("metadata", yaml_content.get(0).unwrap()).unwrap();
    let services_yaml: &Yaml = from_yaml_hash("services", yaml_content.get(0).unwrap()).unwrap();
    let raft_conf_yaml: &Yaml = from_yaml_hash("raft", services_yaml).unwrap();
    let store_conf_yaml: &Yaml = from_yaml_hash("store", services_yaml).unwrap();

    // Metadata
    let node_name = from_yaml_hash("node_name", metadata)
        .and_then(|yaml_string| yaml_enum_to_string(yaml_string))?;

    let listener_addr = from_yaml_hash("listiner_addr", raft_conf_yaml)
        .and_then(|addr_string| yaml_enum_to_string(addr_string))
        .and_then(|sock_addr| {
            let sock_v4 = SocketAddrV4::from_str(&sock_addr)?;
            Ok(SocketAddr::V4(sock_v4))
        })?;

    let connections = from_yaml_hash("connections", raft_conf_yaml)
        .and_then(|s| yaml_to_vec_string(s))?
        .into_iter()
        .map(|ip_string| {
            let ip_addr: &str = ip_string.as_ref();
            return SocketAddr::V4(SocketAddrV4::from_str(ip_addr).unwrap());
        })
        .collect();

    // Store Config
    let local_path_string =
        from_yaml_hash("local_path", store_conf_yaml).and_then(|s| yaml_enum_to_string(s))?;

    let local_path = PathBuf::from(local_path_string);

    let raft_conf: RaftConfig = RaftConfig {
        listener_addr,
        connections,
    };

    let store_conf: StoreConfig = StoreConfig { local_path };

    let conf: Config = Config {
        node_name,
        raft: raft_conf,
        store: store_conf,
    };

    return Ok(conf);

    // return conf;
}

fn from_yaml_hash<'a>(key: &'a str, yaml_content: &'a Yaml) -> Result<&'a Yaml, HelperErrors> {
    // dbg!(yaml_content, key);
    match yaml_content {
        Yaml::Null => Err(HelperErrors::ParserYamlError),
        Yaml::Hash(val) => {
            let result = &val[&Yaml::String(String::from_str(key).unwrap())];
            Ok(result)
        }
        _ => return Err(HelperErrors::ParserYamlError),
    }
}

//
// NOTE: VERY Bad function names
//

fn yaml_enum_to_string(yaml_inp: &Yaml) -> Result<String, HelperErrors> {
    if let Yaml::String(val) = yaml_inp {
        Ok(val.to_owned())
    } else {
        Err(HelperErrors::ParserYamlError)
    }
}

fn yaml_to_vec_string(yaml_inp: &Yaml) -> Result<Vec<String>, HelperErrors> {
    let mut ret_vec: Vec<String> = Vec::new();
    if let Yaml::Array(yaml_arr) = yaml_inp {
        for yaml_val in yaml_arr {
            if let Yaml::String(string_val) = yaml_val {
                // NOTE: is to_owned correct?
                ret_vec.push(string_val.to_owned());
            } else {
                return Err(HelperErrors::ParserYamlError);
            }
        }
    } else {
        return Err(HelperErrors::ParserYamlError);
    }
    Ok(ret_vec)
}