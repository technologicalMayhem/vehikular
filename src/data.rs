pub struct Registration {
    // From registration A
    pub issuer_state: String,
    pub issuer_authority: String,
    pub document_number: String,
    pub registration_number: String,
    pub date_of_first_registration: String,
    pub personal_data: PersonalData,
    pub vehicle: Vehicle,
    pub vehicle_identification_number: String,
    pub mass: Mass,
    pub vehicle_mass_with_body: String,
    pub period_of_validity: String,
    pub date_of_registration: String,
    pub type_approval_number: String,
    pub engine: Engine,
    pub power_weight_ratio: String,
    pub seating_capacity: SeatingCapacity,
    // From registration B
    pub vechicle_category: String,
    pub maximum_towable_mass: MaximumTowableMass,
    pub colour: String,
    pub maximum_speed: String,
    pub exhaust_emissions: ExhaustEmisions,
}

pub struct PersonalData {
    pub certificate_holder: CertificateHolder,
    pub vehicles_owner: Option<bool>
}

pub struct CertificateHolder {
    pub surname_or_business_name: String,
    pub other_name_or_initials: String,
    pub address: String,
}

pub struct Vehicle {
    pub make: String,
    pub vehicle_type: String,
    pub commercial_descriptons: String,
}

pub struct Mass {
    // Registration A
    pub max_permissiable_mass: String,
    // Registration B
    pub max_whole_vehicle_permissable_mass: String,
}

pub struct Engine {
    pub capacity: String,
    pub max_net_power: String,
    pub fuel_type: String,
}

pub struct SeatingCapacity {
    pub number_of_seats: String,
    pub nunmber_of_standing_places: String,
}

pub struct MaximumTowableMass {
    pub braked: String,
    pub unbraked: String,
}

pub struct ExhaustEmisions {
    pub environmental_category: String,
}