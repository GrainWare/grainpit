use std::convert::Infallible;

use chrono::DateTime;
use ipnet::IpNet;
use minicbor_serde::error::{DecodeError, EncodeError};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    pub time: DateTime<chrono::Utc>,
    pub url: String,
    pub ip: IpNet,
    pub user_agent: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Submission {
    pub requests: Vec<Request>,
}

impl Submission {
    pub fn new(requests: Vec<Request>) -> Self {
        Self { requests }
    }

    pub fn serialize(&self) -> Result<Vec<u8>, EncodeError<Infallible>> {
        minicbor_serde::to_vec(self)
    }

    pub fn deserialize(data: Vec<u8>) -> Result<Self, DecodeError> {
        minicbor_serde::from_slice(&data)
    }
}
