use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Model {
    pub id: i32,
    pub issuer_state: String,
    pub issuer_authority: String,
    pub document_number: String,
    pub registration_number: String,
    pub date_of_first_registration: String,
    pub vehicle_identification_number: String,
    pub vehicle_mass_with_body: String,
    pub period_of_validity: String,
    pub date_of_registration: String,
    pub type_approval_number: String,
    pub power_weight_ratio: String,
    pub vechicle_category: String,
    pub colour: String,
    pub maximum_speed: String,
    pub vehicles_owner: String,
    pub surname_or_business_name: String,
    pub other_name_or_initials: String,
    pub address: String,
    pub make: String,
    pub vehicle_type: String,
    pub commercial_descriptons: String,
    pub maximum_technically_laden_mass: String,
    pub maximum_laden_mass_of_the_vehicle_in_service: String,
    pub maximum_laden_mass_of_the_whole_vehicle_in_service: String,
    pub capacity: String,
    pub max_net_power: String,
    pub fuel_type: String,
    pub number_of_seats: String,
    pub nunmber_of_standing_places: String,
    pub braked: String,
    pub unbraked: String,
    pub environmental_category: String,
}
