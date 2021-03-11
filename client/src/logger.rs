use std::{
    fs::{File, OpenOptions},
    io::{ErrorKind, Write},
    net::IpAddr,
};

pub struct Logger {
    log_file_path: String,
}

impl Logger {
    pub fn new(local_ip: IpAddr) -> Logger {
        let log_file_path = format!("output-{}.log", local_ip);
        Logger::create_log_file(&log_file_path);

        Logger { log_file_path }
    }

    fn create_log_file(log_file_path: &str) {
        match File::create(log_file_path) {
            Ok(_file) => {}
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => {
                eprintln!("{}", e);
                panic!("Failed to create log file");
            }
        }
    }

    pub fn log(&self, content: String) {
        let mut log_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.log_file_path)
            .expect("Failed to open log file");
        log_file
            .write(&content.into_bytes())
            .expect("Failed to write to log");
    }
}
