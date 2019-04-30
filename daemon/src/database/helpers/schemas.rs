/*
 * Copyright 2019 Cargill Incorporated
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * -----------------------------------------------------------------------------
 */

use super::models::GridSchema;
use super::schema::{grid_property_definition, grid_schema};
use super::MAX_BLOCK_NUM;

use diesel::{pg::PgConnection, prelude::*, QueryResult};

pub fn list_grid_schemas(conn: &PgConnection) -> QueryResult<Vec<GridSchema>> {
    grid_schema::table
        .select(grid_schema::all_columns)
        .filter(grid_schema::end_block_num.eq(MAX_BLOCK_NUM))
        .left_join(
            grid_property_definition::table.on(grid_property_definition::schema_name
                .eq(grid_schema::name)
                .and(grid_property_definition::end_block_num.eq(MAX_BLOCK_NUM))),
        )
        .load::<GridSchema>(conn)
}
