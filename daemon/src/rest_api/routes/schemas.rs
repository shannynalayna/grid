// Copyright 2019 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::database::{helpers as db, models::GridSchema};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor};

use actix::{Handler, Message, SyncContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
}

impl GridSchemaSlice {
    pub fn from_schema(schema: &GridSchema) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
        }
    }
}

struct ListGridSchemas;

impl Message for ListGridSchemas {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;
}

impl Handler<ListGridSchemas> for DbExecutor {
    type Result = Result<Vec<GridSchemaSlice>, RestApiResponseError>;

    fn handle(&mut self, _msg: ListGridSchemas, _: &mut SyncContext<Self>) -> Self::Result {
        let fetched_schemas = db::list_grid_schemas(&*self.connection_pool.get()?)?
            .iter()
            .map(|schema| GridSchemaSlice::from_schema(schema))
            .collect();
        Ok(fetched_schemas)
    }
}
