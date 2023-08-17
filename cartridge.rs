use std::fs;
use std::path::Path;

pub enum CartridgeType {
  RomOnly,
}

pub struct Cartridge {
  pub title: String,
  pub cartridge_type: CartridgeType, 
  pub data: std::vec::Vec<u8>,
}

impl Cartridge {
  pub fn load(p: &Path) -> Cartridge {
    let res = fs::read(p);

    let data = res.unwrap();
    
    // We only support ROM-only carts for now.
    //assert_eq!(0x00, data[0x0147]);

    return Cartridge {
      // Title is from 0x0134 to 0x0143 inclusive.
      title: String::from_utf8_lossy(&data[0x0134..0x0144]).into_owned(),
      cartridge_type: CartridgeType::RomOnly,
      data: data,
    };
  }
}