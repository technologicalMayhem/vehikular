use shared::data::{
    CertificateHolder, Engine, ExhaustEmisions, Mass, MaximumTowableMass, PersonalData,
    Registration, SeatingCapacity, Vehicle, VehicleOwner,
};

use super::car_registration::{ActiveModel as CarRegActiveModel, Model as CarRegModel};

impl From<Registration> for CarRegActiveModel {
    fn from(value: Registration) -> Self {
        Self {
            id: sea_orm::ActiveValue::NotSet,
            issuer_state: sea_orm::ActiveValue::Set(value.issuer_state),
            issuer_authority: sea_orm::ActiveValue::Set(value.issuer_authority),
            document_number: sea_orm::ActiveValue::Set(value.document_number),
            registration_number: sea_orm::ActiveValue::Set(value.registration_number),
            date_of_first_registration: sea_orm::ActiveValue::Set(value.date_of_first_registration),
            vehicle_identification_number: sea_orm::ActiveValue::Set(
                value.vehicle_identification_number,
            ),
            vehicle_mass_with_body: sea_orm::ActiveValue::Set(value.vehicle_mass_with_body),
            period_of_validity: sea_orm::ActiveValue::Set(value.period_of_validity),
            date_of_registration: sea_orm::ActiveValue::Set(value.date_of_registration),
            type_approval_number: sea_orm::ActiveValue::Set(value.type_approval_number),
            power_weight_ratio: sea_orm::ActiveValue::Set(value.power_weight_ratio),
            vechicle_category: sea_orm::ActiveValue::Set(value.vechicle_category),
            colour: sea_orm::ActiveValue::Set(value.colour),
            maximum_speed: sea_orm::ActiveValue::Set(value.maximum_speed),
            vehicles_owner: sea_orm::ActiveValue::Set(
                value.personal_data.vehicles_owner.to_string(),
            ),
            surname_or_business_name: sea_orm::ActiveValue::Set(
                value
                    .personal_data
                    .certificate_holder
                    .surname_or_business_name,
            ),
            other_name_or_initials: sea_orm::ActiveValue::Set(
                value
                    .personal_data
                    .certificate_holder
                    .other_name_or_initials,
            ),
            address: sea_orm::ActiveValue::Set(value.personal_data.certificate_holder.address),
            make: sea_orm::ActiveValue::Set(value.vehicle.make),
            vehicle_type: sea_orm::ActiveValue::Set(value.vehicle.vehicle_type),
            commercial_descriptons: sea_orm::ActiveValue::Set(value.vehicle.commercial_descriptons),
            maximum_technically_laden_mass: sea_orm::ActiveValue::Set(
                value.mass.maximum_technically_permissible_laden_mass,
            ),
            maximum_laden_mass_of_the_vehicle_in_service: sea_orm::ActiveValue::Set(
                value
                    .mass
                    .maximum_permissible_laden_mass_of_the_vehicle_in_service,
            ),
            maximum_laden_mass_of_the_whole_vehicle_in_service:
                sea_orm::ActiveValue::Set(
                    value
                        .mass
                        .maximum_permissible_laden_mass_of_the_whole_vehicle_in_service,
                ),
            capacity: sea_orm::ActiveValue::Set(value.engine.capacity),
            max_net_power: sea_orm::ActiveValue::Set(value.engine.max_net_power),
            fuel_type: sea_orm::ActiveValue::Set(value.engine.fuel_type),
            number_of_seats: sea_orm::ActiveValue::Set(value.seating_capacity.number_of_seats),
            nunmber_of_standing_places: sea_orm::ActiveValue::Set(
                value.seating_capacity.nunmber_of_standing_places,
            ),
            braked: sea_orm::ActiveValue::Set(value.maximum_towable_mass.braked),
            unbraked: sea_orm::ActiveValue::Set(value.maximum_towable_mass.unbraked),
            environmental_category: sea_orm::ActiveValue::Set(
                value.exhaust_emissions.environmental_category,
            ),
        }
    }
}

impl TryFrom<CarRegModel> for Registration {
    type Error = shared::data::Error;

    fn try_from(value: CarRegModel) -> Result<Self, Self::Error> {
        Ok(Self {
            issuer_state: value.issuer_state,
            issuer_authority: value.issuer_authority,
            document_number: value.document_number,
            registration_number: value.registration_number,
            date_of_first_registration: value.date_of_first_registration,
            personal_data: PersonalData {
                certificate_holder: CertificateHolder {
                    address: value.address,
                    surname_or_business_name: value.surname_or_business_name,
                    other_name_or_initials: value.other_name_or_initials,
                },
                vehicles_owner: VehicleOwner::try_from(value.vehicles_owner)?,
            },
            vehicle: Vehicle {
                make: value.make,
                vehicle_type: value.vehicle_type,
                commercial_descriptons: value.commercial_descriptons,
            },
            vehicle_identification_number: value.vehicle_identification_number,
            mass: Mass {
                maximum_technically_permissible_laden_mass: value
                    .maximum_technically_laden_mass,
                maximum_permissible_laden_mass_of_the_vehicle_in_service: value
                    .maximum_laden_mass_of_the_vehicle_in_service,
                maximum_permissible_laden_mass_of_the_whole_vehicle_in_service: value
                    .maximum_laden_mass_of_the_whole_vehicle_in_service,
            },
            vehicle_mass_with_body: value.vehicle_mass_with_body,
            period_of_validity: value.period_of_validity,
            date_of_registration: value.date_of_registration,
            type_approval_number: value.type_approval_number,
            engine: Engine {
                capacity: value.capacity,
                max_net_power: value.max_net_power,
                fuel_type: value.fuel_type,
            },
            power_weight_ratio: value.power_weight_ratio,
            seating_capacity: SeatingCapacity {
                number_of_seats: value.number_of_seats,
                nunmber_of_standing_places: value.nunmber_of_standing_places,
            },
            vechicle_category: value.vechicle_category,
            maximum_towable_mass: MaximumTowableMass {
                braked: value.braked,
                unbraked: value.unbraked,
            },
            colour: value.colour,
            maximum_speed: value.maximum_speed,
            exhaust_emissions: ExhaustEmisions {
                environmental_category: value.environmental_category,
            },
        })
    }
}
