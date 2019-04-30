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

use crate::database::models::GridSchema;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaSlice {
    pub name: String,
    pub description: String,
    pub owner: String,
    pub property: GridSchemaProperty,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GridSchemaProperty {
    pub name: String,
    pub data_type: String,
    pub required: bool,
    pub description: String,
    pub number_exponent: i32,
    pub enum_options: Vec<String>,
    pub struct_properties: Vec<String>,
}

impl GridSchemaSlice {
    pub fn from_schema(schema: &GridSchema, property: &GridSchemaProperty) -> Self {
        Self {
            name: schema.name.clone(),
            description: schema.description.clone(),
            owner: schema.owner.clone(),
            property: GridSchemaProperty::from_property(property),
        }
    }
}

impl GridSchemaProperty {
    pub fn from_property(property: &GridSchemaProperty) -> Self {
        Self {
            name: property.name.clone(),
            data_type: property.data_type.clone(),
            required: property.required,
            description: property.description.clone(),
            number_exponent: property.number_exponent.clone(),
            enum_options: property.enum_options.clone(),
            struct_properties: property.struct_properties.clone(),
        }
    }
}
