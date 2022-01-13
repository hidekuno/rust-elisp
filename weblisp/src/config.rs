/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   ref) https://doc.rust-jp.rs/book/second-edition/ch20-00-final-project-a-web-server.html

   hidekuno@gmail.com
*/
use std::error::Error;
use std::fmt::Display;
use std::fmt::Formatter;

#[cfg(not(feature = "all-interface"))]
pub const BIND_ADDRESS: &str = "127.0.0.1:9000";

#[cfg(feature = "all-interface")]
pub const BIND_ADDRESS: &str = "0.0.0.0:9000";

pub const MAX_TRANSACTION: usize = 10000;
pub const DEFAULT_NONBLOK: bool = false;
pub const MAX_CONCURRENCY: usize = 4;

const NON_BLOCK_PARAM: &str = "--nb";
const LIMIT_PARAM: &str = "--limit";
const THREAD_POOL_PARAM: &str = "--tp";
const EPOLL_PARAM: &str = "--epoll";
const THREAD_MAX_PARAM: &str = "-m";
const TRANSACTION_MAX_PARAM: &str = "-c";

#[derive(Debug, Clone)]
struct InvalidOptionError;

impl Display for InvalidOptionError {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "invalid option")
    }
}
impl Error for InvalidOptionError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OperationMode {
    Limit,
    ThreadPool,
    Epoll,
}
enum ParamParse {
    ThreadMaxOn,      // -m
    TransactionMaxOn, // -c
    Off,
}
impl ParamParse {
    fn check_option(arg: &str) -> bool {
        arg == NON_BLOCK_PARAM
            || arg == LIMIT_PARAM
            || arg == THREAD_POOL_PARAM
            || arg == EPOLL_PARAM
            || arg == THREAD_MAX_PARAM
            || arg == TRANSACTION_MAX_PARAM
    }
    fn parse_number(arg: &str, max: usize) -> Result<usize, Box<dyn Error>> {
        let n = match arg.parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(Box::new(InvalidOptionError {})),
        };
        if 0 < n && n <= max {
            Ok(n)
        } else {
            Err(Box::new(InvalidOptionError {}))
        }
    }
}
pub struct Config {
    mode: OperationMode,
    nonblock: bool,
    thread_max: usize,
    transaction_max: usize,
}
impl Config {
    fn new() -> Self {
        Config {
            mode: OperationMode::ThreadPool,
            nonblock: false,
            thread_max: MAX_CONCURRENCY,
            transaction_max: MAX_TRANSACTION,
        }
    }
    pub fn mode(&self) -> OperationMode {
        self.mode
    }
    pub fn is_nonblock(&self) -> bool {
        self.nonblock
    }
    pub fn thread_max(&self) -> usize {
        self.thread_max
    }
    pub fn transaction_max(&self) -> usize {
        self.transaction_max
    }
}
struct OptionStatus(bool, bool);

pub fn parse_arg(args: &[String]) -> Result<Config, Box<dyn Error>> {
    let mut parse = ParamParse::Off;
    let mut config = Config::new();
    let mut mode_count = 0;
    let mut option_status = OptionStatus(false, false);

    for arg in args {
        match parse {
            ParamParse::Off => {
                if arg == NON_BLOCK_PARAM {
                    config.nonblock = true;
                } else if arg == LIMIT_PARAM {
                    config.mode = OperationMode::Limit;
                    mode_count += 1;
                } else if arg == THREAD_POOL_PARAM {
                    config.mode = OperationMode::ThreadPool;
                    mode_count += 1;
                } else if arg == EPOLL_PARAM {
                    config.mode = OperationMode::Epoll;
                    mode_count += 1;
                } else if arg == THREAD_MAX_PARAM {
                    parse = ParamParse::ThreadMaxOn;
                    option_status.0 = true;
                } else if arg == TRANSACTION_MAX_PARAM {
                    parse = ParamParse::TransactionMaxOn;
                    option_status.1 = true;
                } else {
                    return Err(Box::new(InvalidOptionError {}));
                }
            }
            ParamParse::TransactionMaxOn => {
                if ParamParse::check_option(arg) {
                    return Err(Box::new(InvalidOptionError {}));
                }
                config.transaction_max = ParamParse::parse_number(arg, MAX_TRANSACTION)?;
                parse = ParamParse::Off;
            }
            ParamParse::ThreadMaxOn => {
                if ParamParse::check_option(arg) {
                    return Err(Box::new(InvalidOptionError {}));
                }
                config.thread_max = ParamParse::parse_number(arg, MAX_CONCURRENCY)?;
                parse = ParamParse::Off;
            }
        }
    }
    if (1 < mode_count)
        || (config.mode == OperationMode::Epoll && option_status.0)
        || (config.mode != OperationMode::Limit && option_status.1)
        || (config.mode != OperationMode::ThreadPool && config.nonblock)
    {
        return Err(Box::new(InvalidOptionError {}));
    }
    Ok(config)
}
