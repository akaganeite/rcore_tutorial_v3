//#![allow(unused)]
use log::{Log,Record, Level, Metadata,LevelFilter};
struct SimpleLogger;

impl log::Log for SimpleLogger {
   fn enabled(&self, _metadata: &log::Metadata) -> bool {
        true
   }

   fn log(&self, record: &log::Record) {
       if !self.enabled(record.metadata()) {
           return;
       }
       let color=match record.level() {
           Level::Trace=> 90,
           Level::Debug=> 32,
           Level::Info=> 34,
           Level::Warn=> 93,
           Level::Error=> 31,
       };
       println!("\x1b[{}m[{}]:{} -- {}\x1b[0m",//\x1b[31mhello world\x1b[0m
                color,
                record.level(),
                record.target(),
                record.args());
   }
   fn flush(&self) {}
}


pub fn init_logger()
{
    static LOGGER:SimpleLogger=SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    match option_env!("LOG") {
        Some("TRACE")=>log::set_max_level(LevelFilter::Trace),
        Some("DEBUG")=>log::set_max_level(LevelFilter::Debug),
        Some("INFO")=>log::set_max_level(LevelFilter::Info),
        Some("WARN")=>log::set_max_level(LevelFilter::Warn),
        Some("ERROR")=>log::set_max_level(LevelFilter::Error),
        _=>log::set_max_level(LevelFilter::Off),
    }
}