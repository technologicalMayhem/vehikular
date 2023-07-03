#!/bin/bash
sea-orm-cli generate entity --with-serde serialize --database-url 'postgres://vehikular:vehikular@localhost:5432/vehikular' -o web-app/src/database/entities
