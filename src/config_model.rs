use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfigModel {
    pub address_device : String,
    pub address_http : String,
    pub address_forward: String
}


impl ConfigModel {
    fn default() -> Self {
        ConfigModel { 
            address_device:"127.0.0.1:20888".to_owned(),
            address_http:"127.0.0.1:20889".to_owned(),
            address_forward:"127.0.0.1:20890".to_owned()
        }
    }
    pub fn read(path:String) -> Result<ConfigModel, std::io::Error> {

        let bts = match fs::read(&path) {
            Ok(bts) => bts,
            Err(_) => {
                let config = ConfigModel::default();

                config.write(path)?;

                return Ok(config);
            }
        };
        
        let str = std::str::from_utf8(&bts).unwrap();
        let config : ConfigModel = serde_xml_rs::from_str(str).unwrap();
        
        Ok(config)
    }

    fn write(&self, path:String) -> Result<(), std::io::Error> {

        let str = serde_xml_rs::to_string(self).unwrap();
        fs::write(path, str)?;

        Ok(())
    }
}


