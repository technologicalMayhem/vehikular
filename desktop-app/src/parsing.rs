use std::{collections::HashMap};

use shared::data::{
    CertificateHolder, Engine, ExhaustEmisions, Mass, MaximumTowableMass, PersonalData,
    Registration, SeatingCapacity, Vehicle, VehicleOwner,
};
use iso7816_tlv::ber::{
    Tlv,
    Value::{Constructed, Primitive},
};


pub fn combine_registrations(registrations: &Vec<Tlv>) -> Registration {
    let hash_map = tlv_to_hash_map(registrations);
    hash_map_to_registration(&hash_map)
}

fn tlv_to_hash_map<'a>(reg: &'a Vec<Tlv>) -> HashMap<String, &'a Vec<u8>> {
    let mut hash_map: HashMap<String, &'a Vec<u8>> = HashMap::new();

    for tlv in reg {
        add_values(tlv, &mut hash_map);
    }

    hash_map
}

fn add_values<'a>(tlv: &'a Tlv, hash_map: &mut HashMap<String, &'a Vec<u8>>) {
    match tlv.value() {
        Constructed(inner) => {
            for inner_tlv in inner {
                add_values(inner_tlv, hash_map);
            }
        }
        Primitive(prim) => {
            hash_map.insert(hex::encode(tlv.tag().to_bytes()), prim);
        }
    }
}

fn hash_map_to_registration(hash_map: &HashMap<String, &Vec<u8>>) -> Registration {
    Registration {
        issuer_state: hash_map.string("9F33"),
        issuer_authority: hash_map.string("9F35"),
        document_number: hash_map.string("9F38"),
        registration_number: hash_map.string("81"),
        date_of_first_registration: hash_map.string("82"),
        personal_data: PersonalData {
            certificate_holder: CertificateHolder {
                surname_or_business_name: hash_map.string("83"),
                other_name_or_initials: hash_map.string("84"),
                address: hash_map.string("85"),
            },
            vehicles_owner: hash_map.vehicle_owner(),
        },
        vehicle: Vehicle {
            make: hash_map.string("87"),
            vehicle_type: hash_map.string("88"),
            commercial_descriptons: hash_map.string("89"),
        },
        vehicle_identification_number: hash_map.string("8A"),
        mass: Mass {
            maximum_technically_permissible_laden_mass: hash_map.string("8B"),
            maximum_permissible_laden_mass_of_the_vehicle_in_service: hash_map.string("86"),
            maximum_permissible_laden_mass_of_the_whole_vehicle_in_service: hash_map.string("97"),
        },
        vehicle_mass_with_body: hash_map.string("8C"),
        period_of_validity: hash_map.string("8D"),
        date_of_registration: hash_map.string("8E"),
        type_approval_number: hash_map.string("8F"),
        engine: Engine {
            capacity: hash_map.string("90"),
            max_net_power: hash_map.string("91"),
            fuel_type: hash_map.string("92"),
        },
        power_weight_ratio: hash_map.string("93"),
        seating_capacity: SeatingCapacity {
            number_of_seats: hash_map.string("94"),
            nunmber_of_standing_places: hash_map.string("95"),
        },
        vechicle_category: hash_map.string("98"),
        maximum_towable_mass: MaximumTowableMass {
            braked: hash_map.string("9B"),
            unbraked: hash_map.string("9C"),
        },
        colour: hash_map.string("9F24"),
        maximum_speed: hash_map.string("25"),
        exhaust_emissions: ExhaustEmisions {
            environmental_category: hash_map.string("9F32"),
        },
    }
}

trait HashMapToRegistrationHelper {
    fn string(&self, path: &str) -> String;
    fn vehicle_owner(&self) -> VehicleOwner;
}

impl HashMapToRegistrationHelper for HashMap<String, &Vec<u8>> {
    fn string(&self, path: &str) -> String {
        if let Some(bytes) = self.get(&path.to_lowercase()) {
            String::from_utf8_lossy(bytes).to_string()
        } else {
            String::from("Not found")
        }
    }

    fn vehicle_owner(&self) -> VehicleOwner {
        if let Some(val) = self.get("86") {
            if **val == vec![0x00u8] {
                VehicleOwner::Yes
            } else if **val == vec![0x01u8] {
                VehicleOwner::No
            } else {
                VehicleOwner::Unknown
            }
        } else {
            VehicleOwner::Unknown
        }
    }
}
