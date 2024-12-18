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

// Trace log macros, call secondary macro processing logic

#[macro_export]
macro_rules! traces {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Trace, $($arg),*);
    };
    () => {};
}

//Debug log macro, call secondary macro processing logic
#[macro_export]
macro_rules! debugs {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Debug, $($arg),*);
    };
    () => {};
}

//Info log macro, call secondary macro processing logic
#[macro_export]
macro_rules! infos {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Info, $($arg),*);
    };
    () => {};
}

// Error log macro, call secondary macro processing logic
#[macro_export]
macro_rules! warns {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Warn, $($arg),*);
    };
    () => {};
}

// Error log macro, call secondary macro processing logic
#[macro_export]
macro_rules! errors {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Error, $($arg),*);
    };
    () => {};
}

// Fatal log macro, call secondary macro processing logic
#[macro_export]
macro_rules! fatals {
    ($logger:expr, $($arg:expr),+) => {
        $crate::logs_common!($logger, $crate::LEVEL::Fatal, $($arg),*);
    };
    () => {};
}

#[macro_export]
macro_rules! formats {
    ($logger:expr, $level:expr, $($arg:expr),*) => {
        let level:$crate::LEVEL = $level;
        unsafe {
            let log:&mut Arc<Mutex<tklog::sync::Logger>> = $logger;
            let mut logger  = log.lock().unwrap();
            let module = module_path!();
            if logger.get_level(module) <= level {
                let mut file = "";
                let mut line = 0;
                if logger.is_file_line($level,module) {
                    file = file!();
                    line = line!();
                }
                let ss = logger.fmt(module,$level, file, line, format!($($arg),*));
                if !ss.is_empty(){
                    logger.print($level,module,ss);
                }
            }
        }
    };
    () => {};
}

#[macro_export]
macro_rules! logs_common {
    ($logger:expr, $level:expr, $($arg:expr),*) => {
        unsafe {
            let  log:&mut Arc<Mutex<tklog::sync::Logger>> = $logger;
            let mut logger  = log.lock().unwrap();
            let module = module_path!();
            if logger.get_level(module) <= $level {
                let formatted_args: Vec<String> = vec![$(format!("{}", $arg)),*];
                let mut file = "";
                let mut line = 0;
                if logger.is_file_line($level,module) {
                    file = file!();
                    line = line!();
                }
                let msg: String = formatted_args.join(logger.get_separator().as_str());
                let ss = logger.fmt(module,$level, file, line, msg);
                if !ss.is_empty(){
                    logger.print($level,module, ss);
                }
            }
        }
    };
    () => {};
}
