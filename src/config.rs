extern crate serde_derive;

use serde_derive::{Serialize, Deserialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub ffmpeg: String,
    pub ffprob: String,
    pub stage: Stage,
    pub input: Input,
    pub output: Output,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub rectangle: Vec<String>,
    pub episodes: Vec<String>,
    pub title: Title,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Title {
    pub font: String,
    pub prefix: String,
    pub x: String,
    pub y: String,
    pub color: String,
    pub size: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output {
    pub stream_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Stage {
    pub image: String,
    pub width: String,
    pub height: String,
}

pub fn parse_config() -> Config {
  let contents = fs::read_to_string("config.json")
    .expect("读取配置文件失败");
  let config: Config = serde_json::from_str(contents.as_str()).unwrap();
  config
}