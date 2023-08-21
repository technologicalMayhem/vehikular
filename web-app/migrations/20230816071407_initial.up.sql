CREATE TABLE car_registration (
	id serial4 NOT NULL,
	issuer_state varchar NOT NULL,
	issuer_authority varchar NOT NULL,
	document_number varchar NOT NULL,
	registration_number varchar NOT NULL,
	date_of_first_registration varchar NOT NULL,
	vehicle_identification_number varchar NOT NULL,
	vehicle_mass_with_body varchar NOT NULL,
	period_of_validity varchar NOT NULL,
	date_of_registration varchar NOT NULL,
	type_approval_number varchar NOT NULL,
	power_weight_ratio varchar NOT NULL,
	vechicle_category varchar NOT NULL,
	colour varchar NOT NULL,
	maximum_speed varchar NOT NULL,
	vehicles_owner varchar NOT NULL,
	surname_or_business_name varchar NOT NULL,
	other_name_or_initials varchar NOT NULL,
	"address" varchar NOT NULL,
	make varchar NOT NULL,
	vehicle_type varchar NOT NULL,
	commercial_descriptons varchar NOT NULL,
	maximum_technically_laden_mass varchar NOT NULL,
	maximum_laden_mass_of_the_vehicle_in_service varchar NOT NULL,
	maximum_laden_mass_of_the_whole_vehicle_in_service varchar NOT NULL,
	capacity varchar NOT NULL,
	max_net_power varchar NOT NULL,
	fuel_type varchar NOT NULL,
	number_of_seats varchar NOT NULL,
	nunmber_of_standing_places varchar NOT NULL,
	braked varchar NOT NULL,
	unbraked varchar NOT NULL,
	environmental_category varchar NOT NULL,
	CONSTRAINT car_registration_pkey PRIMARY KEY (id)
);

CREATE TABLE "user" (
	id serial4 NOT NULL,
	display_name varchar NOT NULL,
	email varchar NOT NULL,
	password_hash varchar NOT NULL,
	CONSTRAINT user_display_name_key UNIQUE (display_name),
	CONSTRAINT user_pkey PRIMARY KEY (id)
);

CREATE TABLE active_session (
	id serial4 NOT NULL,
	user_id int4 NOT NULL,
	"token" varchar NOT NULL,
	idle_timeout timestamp NOT NULL,
	absolute_timeout timestamp NOT NULL,
	CONSTRAINT active_session_pkey PRIMARY KEY (id),
	CONSTRAINT "fk-activesession-user" FOREIGN KEY (user_id) REFERENCES "user"(id)
);

CREATE TABLE maintenance_history (
	id serial4 NOT NULL,
	car_id int4 NOT NULL,
	date_time timestamp NOT NULL,
	subject varchar NOT NULL,
	body varchar NOT NULL,
	mileage int4 NULL,
	CONSTRAINT maintenance_history_pkey PRIMARY KEY (id),
	CONSTRAINT "fk-history-registration" FOREIGN KEY (car_id) REFERENCES car_registration(id)
);

CREATE TABLE vehicle_notes (
	id serial4 NOT NULL,
	car_id int4 NOT NULL,
	body varchar NOT NULL,
	CONSTRAINT vehicle_notes_pkey PRIMARY KEY (id),
	CONSTRAINT "fk-notes-registration" FOREIGN KEY (car_id) REFERENCES car_registration(id)
);