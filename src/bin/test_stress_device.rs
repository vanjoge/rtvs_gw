use std::{time::Duration, io};
use regex::Regex;
use tokio::{net::TcpStream, time::sleep, io::AsyncWriteExt};

fn print_help(address:&String) {
    log::info!("服务器地址:{}", address);
    log::info!("-h //打印help");
    log::info!("-r //运行tcp客户端")
}


#[tokio::main]
async fn main() {
    let address:String = "127.0.0.1:6379".to_owned();
    let mut cmd = String::new();
    loop {
        io::stdin().read_line(&mut cmd).expect("read line err");
        match &cmd as &str {
            "-h" => {
                print_help(&address);
            },
            "-r" => {
                let reg_r = Regex::new(r"^-r (.+) ").unwrap();
                if let Some(caps) = reg_r.captures(&cmd) {
                    let t1 = caps[1].to_string();
                    let t2 = t1.parse::<u32>().unwrap();
                }
                //run(&address);
            },
            _ => {
                log::info!("unkown cmd, input -h");
            }
        }
    }
}
