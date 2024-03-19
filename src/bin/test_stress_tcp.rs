use std::{time::Duration, io};
use regex::Regex;
use tokio::{net::TcpStream, time::sleep, io::AsyncWriteExt};

fn print_help(address:&String) {
    println!("服务器地址:{}", address);
    println!("-h //打印help");
    println!("-r //运行tcp客户端")
}


#[tokio::main]
async fn main() {

    let address:String = "127.0.0.1:6379".to_owned();
    print_help(&address);

    loop {
        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).expect("read line err");
        let (c, content) = cmd.split_at(2);
        match c {
            "-h" => {
                print_help(&address);
            },
            "-r" => {
                let reg_r = Regex::new(r"^-r (.+) ").unwrap();
                if let Some(caps) = reg_r.captures(&cmd) {
                    let t1 = caps[1].to_string();
                    let t2 = t1.parse::<u32>().unwrap();
                } else {
                    println!("-r 参数错误");
                }
                //run(&address);
            },
            _ => {
                println!("unkown cmd, input -h");
            }
        }
    }
}


fn run(address:&String) {
    let addr = address.clone();
    tokio::spawn(async move{
        for _i in 1..1000 {
            match TcpStream::connect(&addr).await {
                Ok(mut stream) => {
                    sleep(Duration::from_secs(5)).await;
                    let _ = stream.write_all(b"hellow");
                    sleep(Duration::from_secs(60)).await;
                },
                Err(_) => {
                    log::warn!("[test-stress-tcp]connect failed, {}", addr);
                },
            }
        }
    });
}
