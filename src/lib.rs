// Copyright (c) 2024, donnie4w <donnie4w@gmail.com>
// All rights reserved.
// https://github.com/donnie4w/tklog
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    fmt::Debug,
    fs::{self, File},
    io::{self, BufReader, BufWriter, Read, Write},
};

use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike};
use flate2::{
    write::{GzEncoder, ZlibEncoder},
    Compression,
};
use once_cell::sync::Lazy;
use tklog::LEVEL;
use tokio::io::AsyncReadExt;

#[allow(non_snake_case)]
pub mod Async;
pub mod asyncfile;
pub mod asyncmulti;
pub mod handle;
pub mod sync;
pub mod syncfile;
pub mod syncmulti;
#[allow(non_snake_case)]
mod threadPool;

pub enum DateType {
    Date,
    Time,
    Microseconds,
}

#[allow(non_upper_case_globals, non_snake_case)]
pub mod Format {
    pub const Nano: u8 = 0;
    pub const Date: u8 = 1;
    pub const Time: u8 = 2;
    pub const Microseconds: u8 = 4;
    pub const LongFileName: u8 = 8;
    pub const ShortFileName: u8 = 16;
    pub const LevelFlag: u8 = 32;
}

#[derive(PartialEq, PartialOrd, Clone, Copy, Debug)]
enum ErrCode {
    NotFound,
}

impl ErrCode {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

pub struct AA;
impl AA {
    pub fn set_level(&self, level: LEVEL) -> &Self {
        unsafe {
            tklog::synclog.set_level(level);
        }
        self
    }
}

pub const LOG: Lazy<sync::Log> = Lazy::new(|| sync::Log::new());
pub const ASYNC_LOG: Lazy<Async::Log> = Lazy::new(|| Async::Log::new());

#[allow(non_upper_case_globals)]
pub mod tklog {
    use crate::{sync, Async};
    use once_cell::sync::Lazy;

    pub static mut synclog: Lazy<sync::Logger> = Lazy::new(|| sync::Logger::new());
    pub static mut asynclog: Lazy<Async::Logger> = Lazy::new(|| Async::Logger::new());

    #[derive(PartialEq, PartialOrd)]
    pub enum PRINTMODE {
        DELAY,
        PUNCTUAL,
    }

    #[derive(PartialEq, PartialOrd, Clone, Copy)]
    #[repr(u8)]
    pub enum LEVEL {
        Trace = 1,
        Debug = 2,
        Info = 3,
        Warn = 4,
        Error = 5,
        Fatal = 6,
        Off = 7,
    }

    pub enum COLUMN {
        LOGFLAG,
        TIME,
        FILEFLAG,
        COLON,
        MESSAGE,
    }
}

#[derive(Copy, Clone)]
pub enum MODE {
    HOUR,
    DAY,
    MONTH,
}

#[derive(Copy, Clone)]
pub enum CUTMODE {
    TIME,
    SIZE,
}

fn timenow() -> Vec<String> {
    let now: DateTime<Local> = Local::now();
    let full_format = now.format("%Y-%m-%d|%H:%M:%S|%.6f").to_string();
    full_format.split('|').map(|s| s.to_string()).collect()
}

#[allow(dead_code)]
fn zlib(filename: &str) -> io::Result<()> {
    let input_file = File::open(filename)?;
    let mut reader = BufReader::new(input_file);
    let mut input_data = Vec::new();
    reader.read_to_end(&mut input_data)?;
    let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
    e.write_all(&input_data)?;
    let compressed_data = e.finish()?;
    let output_filename = format!("{}.zlib", filename);
    let output_file = File::create(&output_filename)?;
    let mut writer = BufWriter::new(output_file);
    let ack = writer.write_all(&compressed_data);
    if ack.is_ok() {
        let _ = fs::remove_file(filename);
    }
    Ok(())
}

fn gzip(filename: &str) -> io::Result<()> {
    let mut input_file = File::open(filename)?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    io::copy(&mut input_file, &mut encoder)?;
    let compressed_data = encoder.finish()?;
    let output_filename = format!("{}.gz", filename);
    let mut output_file = File::create(&output_filename)?;
    let ack = output_file.write_all(&compressed_data);
    if ack.is_ok() {
        let _ = fs::remove_file(filename);
    }
    Ok(())
}

async fn async_gzip(filename: &str) -> io::Result<()> {
    let mut input_file = tokio::fs::File::open(filename).await?;
    let mut file_content = Vec::new();
    input_file.read_to_end(&mut file_content).await?;
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    let _ = encoder.write_all(&file_content);
    let compressed_data = encoder.finish()?;
    let output_filename = format!("{}.gz", filename);
    let mut output_file = tokio::fs::File::create(output_filename).await?;
    tokio::io::AsyncWriteExt::write_all(&mut output_file, &compressed_data).await?;
    let _ = tokio::fs::remove_file(filename).await?;
    Ok(())
}

fn parse_and_format_log(
    format_str: &str,
    level: &str,
    time: &str,
    file: &str,
    message: &str,
) -> String {
    let mut result = String::new();
    let mut in_placeholder = false;
    let mut placeholder = String::new();

    for c in format_str.chars() {
        if in_placeholder {
            if c == '}' {
                in_placeholder = false;
                match placeholder.as_str() {
                    "level" => result.push_str(level),
                    "time" => result.push_str(time),
                    "file" => result.push_str(file),
                    "message" => result.push_str(message),
                    _ => (),
                }
                placeholder.clear();
            } else {
                placeholder.push(c);
            }
        } else if c == '{' {
            in_placeholder = true;
        } else {
            result.push(c);
        }
    }
    result
}

fn getbackup_with_time(startsec: u64, timemode: MODE) -> String {
    let start_time = DateTime::from_timestamp(startsec as i64, 0).expect("");
    match timemode {
        MODE::HOUR => {
            let formatted_time = start_time.format("%Y%m%d%H");
            formatted_time.to_string()
        }
        MODE::DAY => {
            let formatted_time = start_time.format("%Y%m%d");
            formatted_time.to_string()
        }
        MODE::MONTH => {
            let formatted_date = start_time.format("%Y%m");
            formatted_date.to_string()
        }
    }
}

fn get_short_file_path(filename: &str) -> &str {
    let mut pos = None;
    for (i, c) in filename.char_indices().rev() {
        if c == '\\' || c == '/' {
            pos = Some(i);
            break;
        }
    }
    match pos {
        Some(index) => &filename[index + 1..],
        None => filename,
    }
}

fn timesec() -> u64 {
    let now: NaiveDateTime = Local::now().naive_local();
    return now.and_utc().timestamp().try_into().unwrap();
}

fn passtimemode(startsec: u64, timemode: MODE) -> bool {
    let start_time = DateTime::from_timestamp(startsec as i64, 0).expect("");
    let now: NaiveDateTime = Local::now().naive_local();
    match timemode {
        MODE::HOUR => return now.hour() > start_time.hour(),
        MODE::DAY => {
            return now.day() > start_time.day();
        }
        MODE::MONTH => return now.month() > start_time.month(),
    }
}