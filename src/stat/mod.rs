//! 读取 linux 下的 `/proc/stat` 文件, 来获取 cpu、磁盘、内存 使用率


use std::{io, thread};
use std::fs::File;
use std::io::Read;
use std::process::Command;
use std::time::Duration;
use procfs::{Current, Meminfo};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::error::Error;
use crate::monitor::{Monitor, Os, OsDisk};
use crate::prepare::{get_error_response, get_success_response, HttpResponse};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub(crate) os: Os,
    #[serde(rename = "diskInfo")]
    pub(crate) disk_list: Vec<OsDisk>,
    #[serde(rename = "cpuInfo")]
    pub(crate) cpu_info: CpuInfo,
    #[serde(rename = "memInfo")]
    pub(crate) mem_info: MemInfo,
    #[serde(rename = "homeDir")]
    home_dir: String,       // 用户主目录
}

/// 内存信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct MemInfo {
    pub mem_total: u64, // 可用的总物理内存
    pub mem_available: u64 // 估计的可用内存
}

/// CPU 信息
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub physics_core_num: u64, // 物理核心数
    pub virtual_core_num: u64, // 虚拟核心数(包括 `超线程技术（Hyper-Threading）导致的虚拟核心`)
    pub usage: f64, // 使用率
}


pub struct Stat;

impl Stat {

    /// 获取系统信息
    pub(crate) fn get_system_info(cpu_usage: f64) -> HttpResponse {
        let mut system_info = SystemInfo::default();
        let mut monitor = Monitor::new();

        // 系统信息
        let os = monitor.get_system_info();
        system_info.os = os;

        // 磁盘信息
        let disk_list = monitor.get_all_disk_info();
        system_info.disk_list = disk_list;

        let cpu_info = match Self::get_cup_info() {
            Ok(info) => {
                info
            }
            Err(err) => {
                return Self::get_system_info_error(&system_info, &err);
            }
        };

        system_info.cpu_info = cpu_info;
        system_info.cpu_info.usage = cpu_usage;

        let mem_info = match Self::get_mem_info() {
            Ok(info) => {
                info
            }
            Err(err) => {
                return Self::get_system_info_error(&system_info, &err);
            }
        };

        system_info.mem_info = mem_info;

        let home_dir = Self::get_user_home_dir();
        system_info.home_dir = home_dir;
        let data = serde_json::to_value(&system_info).unwrap_or(Value::default());
        return get_success_response(Some(data))
    }

    /// 获取用户主目录
    fn get_user_home_dir() -> String {
        let username = Self::get_current_login_username();
        if username.is_empty() {
            return String::new()
        }

        let command = format!("getent passwd {}", &username);
        let content = Self::get_command_output(&command);
        if !content.is_empty() {
            let fields: Vec<&str> = content.trim().split(':').collect();
            if fields.len() >= 6 {
                return fields[5].to_string()
            }
        }
        return String::new()
    }

    fn get_current_login_username() -> String {
        let content = Self::get_command_output("echo $USER");
        if content.is_empty() {
            return String::new()
        }

        return content;
    }

    fn get_command_output(command: &str) -> String {
        let output = Command::new("sh").arg("-c").arg(command).output();
        return match output {
            Ok(output) => {
                if output.status.success() {
                    let content = String::from_utf8_lossy(&output.stdout).to_string();
                    return content
                }

                return String::new()
            },
            Err(_) => {
                String::new()
            }
        }
    }

    /// 获取内存使用情况
    fn get_mem_info() -> Result<MemInfo, String> {
        let mem_info = Meminfo::current().map_err(|err| Error::Error(err.to_string()).to_string())?;
        let mem_total = mem_info.mem_total;
        let mem_available = mem_info.mem_available.unwrap_or(0);
        Ok(MemInfo {
            mem_total,
            mem_available,
        })
    }

    fn get_cup_info() -> Result<CpuInfo, String> {
        let cup_info = procfs::CpuInfo::current().map_err(|err| Error::convert_string(err.to_string().as_str()))?;
        let fields = cup_info.fields.clone();
        let cpus = cup_info.cpus.clone();

        let mut info = CpuInfo::default();
        if fields.len() > 0 {
            info.physics_core_num = fields.get("cpu cores").unwrap_or(&"0".to_string()).parse::<u64>().unwrap_or(0);
        }

        if cpus.len() > 0 {
            info.virtual_core_num = cpus.len() as u64
        }
        Ok(info)

    }

    /// 返回错误
    fn get_system_info_error(system_info: &SystemInfo, err: &str) -> HttpResponse {
        let mut res = get_error_response(err);
        let body = serde_json::to_value(&system_info).unwrap_or(Value::default());
        res.body = body;
        return res
    }
}


/// MARK: 计算 CPU 使用率
/**!
只看行首以cpu开头的行，每列字段含义为：
    * name: 设备名
    * user: 从系统启动开始累计到当前时刻，处于用户态的运行时间，不包含 nice值为负进程。
    * nice: 从系统启动开始累计到当前时刻，nice值为负的进程所占用的CPU时间。
    * system: 从系统启动开始累计到当前时刻，处于核心态的运行时间。
    * idle: 从系统启动开始累计到当前时刻，除IO等待时间以外的其它等待时间。
    * iowait: 从系统启动开始累计到当前时刻，IO等待时间。
    * irq: 从系统启动开始累计到当前时刻，硬中断时间。
    * softirq: 从系统启动开始累计到当前时刻，软中断时间。
    * stealstolen: 从系统启动开始累积到当前时刻，在虚拟环境运行时花费在其他操作系统的时间。
    * guest: 从系统启动开始累积到当前时刻，在Linux内核控制下的操作系统虚拟cpu花费的时间。
    * guest_nice: 从系统启动开始累积到当前时刻，在Linux内核控制下的操作系统虚拟cpu花费在nice进程上的时间


计算方式：
    CPU 总时间 = user + nice + system + idle + iowait + irq + softirq + stealstolen + guest + guest_nice
    CPU 使用率计算
     1. 请在一段时间内（推荐：必须大于0s，小于等于1s），获取两次 CPU 时间分配信息。
     2. 计算两次的 CPU 总时间：total_2 - total_1
     3. 计算两次的 CPU 剩余时间：idle_2 - idle_1
     4. 计算两次的 CPU 使用时间：used = (total_2 - total_1) - (idle_2 - idle_1)
     5. CPU 使用率 = 使用时间 / 总时间 * 100% = used / total * 100%
 */

#[derive(Debug, Clone)]
pub struct CpuUsageInfo {
    user: u64,
    nice: u64,
    system: u64,
    idle: u64,
    io_wait: u64,
    irq: u64,
    soft_irq: u64,
    steal_stolen: u64,
    guest: u64,
    guest_nice: u64
}

impl CpuUsageInfo {
    pub(crate) fn get_total_time(info: &CpuUsageInfo) -> u64 {
        info.user + info.nice + info.system + info.idle + info.io_wait + info.irq + info.soft_irq + info.steal_stolen + info.guest + info.guest_nice
    }
}

impl Stat {

    /// 获取 CPU 信息
    #[allow(dead_code)]
    pub(crate) fn get_cup_used() {
        let mut prev_stat = Self::read_proc_stat().unwrap();
        loop {
            thread::sleep(Duration::from_secs(1));
            let current_stat = Self::read_proc_stat().unwrap();

            let cpu_usage = Self::calculate_cpu_usage(prev_stat, current_stat.clone());
            println!("CPU Usage: {:.2}%", cpu_usage);
            prev_stat = current_stat.clone();
        }
    }

    /// 读取 `/proc/stat`
    pub(crate) fn read_proc_stat() -> Result<CpuUsageInfo, io::Error> {
        let mut file = File::open("/proc/stat")?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let lines: Vec<&str> = contents.lines().collect();
        let cpu_line = lines.iter().find(|line| line.starts_with("cpu ")).unwrap();
        let fields: Vec<&str> = cpu_line.split_whitespace().collect();

        let user: u64 = fields[1].parse().unwrap();
        let nice: u64 = fields[2].parse().unwrap();
        let system: u64 = fields[3].parse().unwrap();
        let idle: u64 = fields[4].parse().unwrap();
        let io_wait: u64 = fields[5].parse().unwrap();
        let irq: u64 = fields[6].parse().unwrap();
        let soft_irq: u64 = fields[7].parse().unwrap();
        let steal_stolen: u64 = fields[8].parse().unwrap();
        let guest: u64 = fields[9].parse().unwrap();
        let guest_nice: u64 = fields[10].parse().unwrap();

        Ok(CpuUsageInfo {
            user,
            nice,
            system,
            idle,
            io_wait,
            irq,
            soft_irq,
            steal_stolen,
            guest,
            guest_nice
        })
    }

    /// 计算
    pub(crate) fn calculate_cpu_usage(current: CpuUsageInfo, prev: CpuUsageInfo) -> f64 {
        // println!("current: {:?}", current);
        // println!("prev: {:?}", prev);

        // 1. 计算两次的 CPU 总时间
        let current_total_time = CpuUsageInfo::get_total_time(&current); // 当前总时间
        let prev_total_time = CpuUsageInfo::get_total_time(&prev); // 前一个时间段的总时间
        // println!("current_total_time: {}", current_total_time);
        //  println!("prev_total_time: {}", prev_total_time);

        // 2. 计算两次的 CPU 剩余时间
        let left_time = current.idle - prev.idle;
        // println!("left_time: {}", left_time);

        // 3. 计算两次的 CPU 使用时间
        let usage_time = (current_total_time - prev_total_time) - left_time;
        // println!("usage_time: {}", usage_time);

        // 4. 总时间
        let usage_total_time = current_total_time - prev_total_time;
        // println!("usage_total_time: {}", usage_total_time);

        // 4. 计算 CPU 使用率: usage_time/usage_total_time
        if usage_time > 0 && usage_total_time > 0 {
            let cpu_usage: f64 = (usage_time as f64 / usage_total_time as f64) * 100.0;
            // println!("cpu_usage: {}", cpu_usage);
            let cpu_usage_str = format!("{:.2}", cpu_usage);
            // println!("cpu_usage format: {}", cpu_usage_str);
            return cpu_usage_str.parse::<f64>().unwrap();
        }

        return 0.0;
    }
}