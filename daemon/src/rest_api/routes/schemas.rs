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

use crate::database::{
    helpers as db,
    models::{GridPropertyDefinition, GridSchema},
};
use crate::rest_api::{error::RestApiResponseError, routes::DbExecutor, AppState};

use actix::{Handler, Message, SyncContext};
use actix_web::{AsyncResponder, HttpRequest, HttpResponse};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub property: Vec<GridSchemaProperty>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaProperty {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i64,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

impl GridSchemaSlice {
    pub fn from_schema(schema: &GridSchema, property: Vec<GridPropertyDefinition>) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            property: property
                .iter()
                .map(|prop| GridSchemaProperty::from(prop))
                .collect::<Vec<GridSchemaProperty>>(),
        }
    }
}

impl GridSchemaProperty {
    pub fn from(property: &GridPropertyDefinition) -> Self {
        Self {
            name: property.name.clone(),
            data_type: property.data_type.clone(),
            required: property.required,
            description: property.description.clone(),
            number_exponent: property.number_exponent,
            enum_options: property.enum_options.clone(),
            struct_properties: property.struct_properties.clone(),
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
        let mut fetched_definitions =
            db::list_grid_property_definitions(&*self.connection_pool.get()?)?
                .into_iter()
                .fold(HashMap::new(), |mut acc, def| {
                    acc.entry(def.schema_name.to_string())
                        .or_insert_with(|| vec![])
                        .push(def);
                    acc
                });

        let fetched_schemas = db::list_grid_schemas(&*self.connection_pool.get()?)?
            .iter()
            .map(|schema| {
                GridSchemaSlice::from_schema(
                    schema,
                    fetched_definitions
                        .remove(&schema.name)
                        .unwrap_or_else(|| vec![]),
                )
            })
            .collect();

        Ok(fetched_schemas)
    }
}

pub fn list_grid_schemas(
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = RestApiResponseError>> {
    req.state()
        .database_connection
        .send(ListGridSchemas)
        .from_err()
        .and_then(move |res| match res {
            Ok(schemas) => Ok(HttpResponse::Ok().json(schemas)),
            Err(err) => Err(err),
        })
        .responder()
}
