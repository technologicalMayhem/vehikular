use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Registration {
    // From registration A
    pub issuer_state: String, // 9F33
    pub issuer_authority: String, // 9F34
    pub document_number: String, // 9F38
    pub registration_number: String, // 81
    pub date_of_first_registration: String, // 82
    pub personal_data: PersonalData,
    pub vehicle: Vehicle,
    pub vehicle_identification_number: String, // 8A
    pub mass: Mass,
    pub vehicle_mass_with_body: String, // 8C
    pub period_of_validity: String, // 8D
    pub date_of_registration: String, // 8E
    pub type_approval_number: String, // 8F
    pub engine: Engine,
    pub power_weight_ratio: String, // 93
    pub seating_capacity: SeatingCapacity,
    // From registration B
    pub vechicle_category: String, // 98
    pub maximum_towable_mass: MaximumTowableMass,
    pub colour: String, // 9F24
    pub maximum_speed: String, // 25
    pub exhaust_emissions: ExhaustEmisions,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalData {
    pub certificate_holder: CertificateHolder,
    pub vehicles_owner: VehicleOwner // 86
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleOwner {
    Yes,
    No,
    Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateHolder {
    pub surname_or_business_name: String, // 83
    pub other_name_or_initials: String, // 84
    pub address: String, // 85
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub make: String, // 87
    pub vehicle_type: String, // 88
    pub commercial_descriptons: String, // 89
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mass {
    // Registration A
    pub maximum_technically_permissible_laden_mass: String, // 8B
    // Registration B
    pub maximum_permissible_laden_mass_of_the_vehicle_in_service: String, // 86
    pub maximum_permissible_laden_mass_of_the_whole_vehicle_in_service: String, // 97
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Engine {
    pub capacity: String, // 90
    pub max_net_power: String, // 91
    pub fuel_type: String, // 92
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatingCapacity {
    pub number_of_seats: String, // 94
    pub nunmber_of_standing_places: String, // 95
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaximumTowableMass {
    pub braked: String, // 9B
    pub unbraked: String, // 9C
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExhaustEmisions {
    pub environmental_category: String, // 9F31
}