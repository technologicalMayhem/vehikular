use std::{collections::HashMap, fs};

use card_reader::data::Registration;
use color_eyre::Result;
use iso7816_tlv::ber::{
    Tlv,
    Value::{Constructed, Primitive},
};

fn main() -> Result<()> {
    let reg_a = fs::read("RegistrationA.data")?;
    let reg_b = fs::read("RegistrationB.data")?;
    let reg_c = fs::read("RegistrationC.data")?;

    let reg_a = Tlv::parse_all(&reg_a);
    let reg_b = Tlv::parse_all(&reg_b);
    let reg_c = Tlv::parse_all(&reg_c);

    let full_reg = [reg_a, reg_b, reg_c].concat();

    let hash_map = tlv_to_hash_map(&full_reg);

    let registration = hash_map_to_registration(&hash_map);

    println!("{hash_map:#?}");

    Ok(())
}

fn tlv_to_hash_map<'a>(reg: &'a Vec<Tlv>) -> HashMap<String, &'a Vec<u8>> {
    let mut hash_map: HashMap<String, &'a Vec<u8>> = HashMap::new();

    for tlv in reg {
        add_values(tlv, &mut hash_map, "");
    }

    hash_map
}

fn add_values<'a>(tlv: &'a Tlv, hash_map: &mut HashMap<String, &'a Vec<u8>>, prefix: &str) {
    let tag = hex::encode(tlv.tag().to_bytes());
    match tlv.value() {
        Constructed(inner) => {
            for inner_tlv in inner {
                add_values(inner_tlv, hash_map, &format!("{prefix}{tag}/"));
            }
        }
        Primitive(prim) => {
            hash_map.insert(format!("{prefix}{tag}",), prim);
        }
    }
}

fn hash_map_to_registration(hash_map: &HashMap<String, &Vec<u8>>) -> Registration {
    Registration {
        issuer_state: hash_map.string("71/9F33"),
        issuer_authority: hash_map.string("71/9f35"),
        document_number: (),
        registration_number: (),
        date_of_first_registration: (),
        personal_data: (),
        vehicle: (),
        vehicle_identification_number: (),
        mass: (),
        vehicle_mass_with_body: (),
        period_of_validity: (),
        date_of_registration: (),
        type_approval_number: (),
        engine: (),
        power_weight_ratio: (),
        seating_capacity: (),
        vechicle_category: (),
        maximum_towable_mass: (),
        colour: (),
        maximum_speed: (),
        exhaust_emissions: (),
    }
}

trait HashMapToRegistrationHelper {
    fn string(&self, path: &str) -> String;
}

impl HashMapToRegistrationHelper for HashMap<String, &Vec<u8>> {
    fn string(&self, path: &str) -> String {
        if let Some(bytes) = self.get(path) {
            String::from_utf8_lossy(&bytes)
        } else {
            String::from("Not found")
        }
    }
}


