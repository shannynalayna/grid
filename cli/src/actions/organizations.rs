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

use std::time::{SystemTime, UNIX_EPOCH};

use cylinder::Signer;
use grid_sdk::{
    pike::addressing::PIKE_NAMESPACE,
    protocol::pike::payload::{
        Action, CreateOrganizationAction, PikePayloadBuilder, UpdateOrganizationAction,
    },
    protos::IntoProto,
};
use reqwest::Client;
use serde::Deserialize;

use crate::actions::Paging;
use crate::error::CliError;
use crate::http::submit_batches;
use crate::transaction::pike_batch_builder;

#[derive(Debug, Deserialize)]
pub struct AlternateIdSlice {
    pub id_type: String,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationMetadataSlice {
    pub key: String,
    pub value: String,
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationSlice {
    pub org_id: String,
    pub name: String,
    pub locations: Vec<String>,
    pub alternate_ids: Vec<AlternateIdSlice>,
    pub metadata: Vec<OrganizationMetadataSlice>,
    pub service_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizationListSlice {
    pub data: Vec<OrganizationSlice>,
    pub paging: Paging,
}

pub fn do_create_organization(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    create_org: CreateOrganizationAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::CreateOrganization(create_org))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_update_organization(
    url: &str,
    signer: Box<dyn Signer>,
    wait: u64,
    update_org: UpdateOrganizationAction,
    service_id: Option<String>,
) -> Result<(), CliError> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .map_err(|err| CliError::PayloadError(format!("{}", err)))?;

    let payload = PikePayloadBuilder::new()
        .with_action(Action::UpdateOrganization(update_org))
        .with_timestamp(timestamp)
        .build()
        .map_err(|err| CliError::UserError(format!("{}", err)))?;

    let batch_list = pike_batch_builder(signer)
        .add_transaction(
            &payload.into_proto()?,
            &[PIKE_NAMESPACE.to_string()],
            &[PIKE_NAMESPACE.to_string()],
        )?
        .create_batch_list();

    submit_batches(url, wait, &batch_list, service_id.as_deref())
}

pub fn do_list_organizations(
    url: &str,
    service_id: Option<String>,
    format: &str,
    display_alternate_ids: bool,
) -> Result<(), CliError> {
    let client = Client::new();
    let mut final_url = format!("{}/organization", url);
    if let Some(service_id) = service_id {
        final_url = format!("{}?service_id={}", final_url, service_id);
    }

    let mut orgs = Vec::new();

    loop {
        let mut response = client.get(&final_url).send()?;

        if !response.status().is_success() {
            return Err(CliError::DaemonError(response.text()?));
        }

        let mut orgs_list = response.json::<OrganizationListSlice>()?;
        orgs.append(&mut orgs_list.data);

        if let Some(next) = orgs_list.paging.next {
            final_url = format!("{}{}", url, next);
        } else {
            break;
        }
    }

    list_organizations(orgs, format, display_alternate_ids);
    Ok(())
}

fn list_organizations(orgs: Vec<OrganizationSlice>, format: &str, display_alternate_ids: bool) {
    let mut rows = vec![];
    let mut headers = vec![
        "ORG_ID".to_string(),
        "NAME".to_string(),
        "LOCATIONS".to_string(),
    ];
    if display_alternate_ids {
        headers.push("ALTERNATE_IDS".to_string());
    }
    rows.push(headers);
    orgs.iter().for_each(|org| {
        let mut values = vec![
            org.org_id.to_string(),
            org.name.to_string(),
            org.locations.join(", "),
        ];
        if display_alternate_ids {
            values.push(
                org.alternate_ids
                    .iter()
                    .map(|id| format!("{}:{}", id.id_type, id.id))
                    .collect::<Vec<String>>()
                    .join(", "),
            );
        }
        rows.push(values);
    });
    if format == "csv" {
        for row in rows {
            print!("{}", row.join(","))
        }
    } else {
        print_table(rows);
    }
}

// Takes a vec of vecs of strings. The first vec should include the title of the columns.
// The max length of each column is calculated and is used as the column with when printing the
// table.
fn print_table(table: Vec<Vec<String>>) {
    let mut max_lengths = Vec::new();

    // find the max lengths of the columns
    for row in table.iter() {
        for (i, col) in row.iter().enumerate() {
            if let Some(length) = max_lengths.get_mut(i) {
                if col.len() > *length {
                    *length = col.len()
                }
            } else {
                max_lengths.push(col.len())
            }
        }
    }

    // print each row with correct column size
    for row in table.iter() {
        let mut col_string = String::from("");
        for (i, len) in max_lengths.iter().enumerate() {
            if let Some(value) = row.get(i) {
                col_string += &format!("{}{} ", value, " ".repeat(*len - value.len()),);
            } else {
                col_string += &" ".repeat(*len);
            }
        }
        println!("{}", col_string);
    }
}
