//! 通过 sysinfo 来获取系统信息

use serde::{Deserialize, Serialize};
use sysinfo::{DiskExt, DiskKind, System, SystemExt};

/// 系统信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Os {
    os_type: String,        // 名称(平台类型)
    kernel_version: String, // 内核版本
    os_version: String,     // 操作系统版本
    host_name: String,      // 系统名称
    cpu_num: usize,         // CPU 总核数
    total_memory: u64,      // 总内存
    used_memory: u64,       // 忆使用内存
    total_swap: u64,        // 总交换分区
    used_swap: u64,         // 已使用的交换分区
}

/// 磁盘信息
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OsDisk {
    type_: String, // 磁盘类型
    name: String,
    total_space: u64,     // 总数
    available_space: u64, // 已使用数
    mount_point: String,
    is_removable: bool, // 是否被移除
}

/// 磁盘使用情况(相对于进程)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskUsage {
    pub total_written_bytes: u64, // 总写入数
    pub written_bytes: u64,       // 写入数
    pub total_read_bytes: u64,    // 总读取数
    pub read_bytes: u64,          // 读取数
}


pub struct Monitor {
    sys: System
}

impl Monitor {
    pub fn new() -> Self {
        Self { sys: System::new_all() }
    }

    /// 获取系统信息
    pub(crate) fn get_system_info(&mut self) -> Os {
        self.sys.refresh_all();

        return Os {
            os_type: self.sys.name().unwrap_or(String::new()),
            kernel_version: self.sys.kernel_version().unwrap_or(String::new()),
            os_version: self.sys.os_version().unwrap_or(String::new()),
            host_name: self.sys.host_name().unwrap_or(String::new()),
            cpu_num: self.sys.cpus().len(),
            total_memory: self.sys.total_memory(),
            used_memory: self.sys.used_memory(),
            total_swap: self.sys.total_swap(),
            used_swap: self.sys.used_swap(),
        };
    }

    /// 获取所有磁盘信息
    pub fn get_all_disk_info(&mut self) -> Vec<OsDisk> {
        self.sys.refresh_all();
        let mut disks: Vec<OsDisk> = Vec::new();

        for disk in self.sys.disks() {
            let type_ = match disk.kind() {
                DiskKind::HDD => "HDD",
                DiskKind::SSD => "SSD",
                DiskKind::Unknown(_) => "UNKNOWN",
            };
            disks.push(OsDisk {
                type_: String::from(type_),
                name: String::from(disk.name().to_str().unwrap_or("")),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                mount_point: String::from(disk.mount_point().to_owned().to_str().unwrap_or("")),
                is_removable: disk.is_removable(),
            });
        }

        return disks;
    }

}