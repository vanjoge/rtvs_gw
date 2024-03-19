use std::{time::Duration, sync::Arc};

mod session808;
mod session_forward;

mod config_model;
mod service_device;
mod service_http;
mod service_forward;


#[tokio::main]
async fn main() {
    //配置
    let config = config_model::ConfigModel::read("SettingConfig.xml".to_owned()).unwrap();

    //日志
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();

    //启动转发服务
    let fw_service = Arc::new(service_forward::ServiceForward::new());
    let _ = service_forward::ServiceForward::start(fw_service.clone(), &config.address_forward).await;

    //启动设备服务
    service_device::init();
    let _ = service_device::start(&config.address_device, fw_service.clone()).await;

    //启动http服务
    service_http::start(&config.address_http).await;

    //let _ = service_device::send(&"111221122".to_owned());

    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
