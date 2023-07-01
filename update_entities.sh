#!/bin/bash
sea-orm-cli generate entity --with-serde serialize --database-url 'postgres://vehikular:vehikular@localhost:5432/vehikular' -o web-app/src/entities
echo "
pub mod convert;" >> web-app/src/entities/mod.rs
