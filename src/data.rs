struct Registration {
    // From registration A
    issuer_state: String,
    issuer_authority: String,
    document_number: String,
    registration_number: String,
    date_of_first_registration: String,
    personal_data: PersonalData,
    vehicle: Vehicle,
    vehicle_identification_number: String,
    mass: Mass,
    vehicle_mass_with_body: String,
    period_of_validity: String,
    date_of_registration: String,
    type_approval_number: String,
    engine: Engine,
    power_weight_ratio: String,
    seating_capacity: SeatingCapacity,
    // From registration B
    vechicle_category: String,
    maximum_towable_mass: MaximumTowableMass,
    colour: String,
    maximum_speed: String,
    exhaust_emissions: ExhaustEmisions,
}

struct PersonalData {
    certificate_holder: CertificateHolder,
    vehicles_owner: Option<bool>
}

struct CertificateHolder {
    surname_or_business_name: String,
    other_name_or_initials: String,
    address: String,
}

struct Vehicle {
    make: String,
    vehicle_type: String,
    commercial_descriptons: String,
}

struct Mass {
    // Registration A
    max_permissiable_mass: String,
    // Registration B
    max_whole_vehicle_permissable_mass: String,
}

struct Engine {
    capacity: String,
    max_net_power: String,
    fuel_type: String,
}

struct SeatingCapacity {
    number_of_seats: String,
    nunmber_of_standing_places: String,
}

struct MaximumTowableMass {
    braked: String,
    unbraked: String,
}

struct ExhaustEmisions {
    environmental_category: String,
}