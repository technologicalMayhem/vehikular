use std::{collections::HashMap, fs};

use color_eyre::Result;
use iso7816_tlv::ber::{
    Tlv,
    Value::{Constructed, Primitive},
};
use once_cell::sync::Lazy;

fn main() -> Result<()> {
    let data = fs::read("RegistrationA.data")?;

    let tlvs = Tlv::parse_all(&data);
    for tlv in tlvs {
        read_out(&tlv, 0);
    }

    Ok(())
}

static FIELD_NAMES: Lazy<HashMap<&str, &str>> = Lazy::new(|| {
    HashMap::from([
        ("9F33", "Lidstaat van uitgifte"),
        ("9F35", "Authority"),
        ("9F38", "Documentnummer"),
        ("81", "A Registration number"),
        ("82", "B Date of first registration"),
        ("A1", "C Personal data"),
        ("A2", "C.1 Holder of the registration certificate"),
        ("83", "C.1.1 Surname or business name"),
        ("84", "C.1.2 Other names or initials"),
        ("85", "C.1.3 Address in the Member State"),
        ("86", "C.4 vehicle owner yes/no/unknown"),
    ])
});

fn read_out(tlv: &Tlv, depth: u32) {
    let tag_hex = hex::encode(tlv.tag().to_bytes()).to_uppercase();
    let tag = *FIELD_NAMES.get(tag_hex.as_str()).unwrap_or(&tag_hex.as_str());
    print_padded(&format!("{tag}:"), depth);

    match tlv.value() {
        Constructed(tlvs) => {
            for tlv in tlvs {
                read_out(tlv, depth + 2);
            }
        }
        Primitive(prim) => {
            if prim.is_empty() {
                print_padded("<null>", depth + 2);
            } else {
                match special_cases(&tag_hex, prim) {
                    Some(s) => print_padded(s, depth + 2),
                    None => print_padded(&String::from_utf8_lossy(prim), depth + 2),
                }
            }
        }
    }
}

fn special_cases(tag_hex: &str, data: &Vec<u8>) -> Option<&'static str> {
    match tag_hex {
        "86" => {
            match hex::encode(data).as_str() {
                "00" => Some("Yes"),
                "01" => Some("No"),
                "02" => Some("Unknown"),
                _ => Some("Unexpected Value")
            }
        }
        _ => None,
    }
}

fn print_padded(str: &str, pad: u32) {
    let mut s = String::new();
    for _ in 0..pad {
        s.push(' ');
    }
    s.push_str(str);
    println!("{s}");
}
