use std::{thread};
use std::time::Duration;
use crate::prepare::{to_result};
use crate::stat::{Stat};

mod stat;
mod error;

mod prepare;
mod monitor;

fn main() {
    let mut prev_stat = Stat::read_proc_stat().unwrap();
    loop {
        thread::sleep(Duration::from_secs(1));
        let current_stat = Stat::read_proc_stat().unwrap();

        let cpu_usage = Stat::calculate_cpu_usage(current_stat.clone(), prev_stat.clone());
        // println!("CPU Usage: {:.2}%", cpu_usage);

        let result = Stat::get_system_info(cpu_usage);
        println!("system info: {}", to_result(result));
        prev_stat = current_stat.clone();
    }
}
