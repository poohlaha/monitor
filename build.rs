//! 设置编译平台

fn main() {
    if cfg!(target_os = "linux") {
        println!("info: Build on Linux!");
    } else {
        println!("warning: This project can only be built on Linux!");
        std::process::exit(1);
    }
}