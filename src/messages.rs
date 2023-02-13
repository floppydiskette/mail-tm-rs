use std::io::Read;
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};

use crate::{http, MAIL_API_URL};
use crate::http::{Client, get_headers};
use crate::hydra::{HydraCollection, Search, View};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Messages {
    #[serde(rename = "hydra:member")]
    pub messages: Vec<Message>,
    #[serde(rename = "hydra:totalItems")]
    pub total_items: i64,
    #[serde(rename = "hydra:view")]
    pub view: Option<View>,
    #[serde(rename = "hydra:search")]
    pub search: Option<Search>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    #[serde(rename = "@context")]
    pub context: String,
    #[serde(rename = "@id")]
    pub id: String,
    #[serde(rename = "@type")]
    pub type_field: String,
    #[serde(rename = "id")]
    pub id2: String,
    #[serde(rename = "account_id")]
    pub account_id: String,
    #[serde(rename = "msgid")]
    pub msg_id: String,
    pub from: From,
    pub to: Vec<To>,
    pub cc: Vec<::serde_json::Value>,
    pub bcc: Vec<::serde_json::Value>,
    pub subject: String,
    pub seen: bool,
    pub flagged: bool,
    #[serde(rename = "verification_results")]
    pub verification_results: Vec<::serde_json::Value>,
    pub retention: bool,
    #[serde(rename = "retention_date")]
    pub retention_date: i64,
    pub text: String,
    pub html: Vec<String>,
    #[serde(rename = "has_attachments")]
    pub has_attachments: bool,
    pub attachments: Vec<::serde_json::Value>,
    #[serde(rename = "download_url")]
    pub download_url: String,
    pub size: i64,
    #[serde(rename = "created_at")]
    pub created_at: String,
    #[serde(rename = "updated_at")]
    pub updated_at: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct From {
    pub address: String,
    pub name: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct To {
    pub address: String,
    pub name: String,
}

pub(crate) fn messages(token: &str, page: Option<usize>) -> Result<HydraCollection<Message>, Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Getting messages");

    let builder = format!("{}/messages", MAIL_API_URL);
    let builder = if let Some(idx) = page {
        builder + &format!("?page={}", idx)
    } else {
        builder
    };

    let mut response = client
        .get(&builder)?;

    let code = response.status();

    let response = {
        let mut buffer = String::new();
        response.body_mut().read_to_string(&mut buffer)?;
        buffer
    };

    http::check_response_status(&code, &response)?;

    log::trace!("Retrieved domains: {}", response);
    Ok(serde_json::from_str(&response)?)
}

pub(crate) fn get(token: &str, id: &str) -> Result<Message, Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Searching for message with id {}", id);


    let mut response = client
        .get(&format!("{}/messages/{}", MAIL_API_URL, id))?;

    let code = response.status();

    let response = {
        let mut buffer = String::new();
        response.body_mut().read_to_string(&mut buffer)?;
        buffer
    };

    http::check_response_status(&code, &response)?;

    log::trace!("Retrieved a message: {}", response);
    Ok(serde_json::from_str(&response)?)
}


pub(crate) fn delete(token: &str, id: &str) -> Result<(), Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Searching for account with id {}", id);


    let response = client
        .delete(&format!("{}/messages/{}", MAIL_API_URL, id))?;

    let code = response.status();

    http::check_response_status(&code, "")?;

    log::trace!("Deleted user with id {}", id);
    Ok(())
}

// TODO impl me
pub(crate) fn patch(token: &str, id: &str) -> Result<(), Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::accounts;
    use crate::accounts::create;
    use crate::user::User;

    use super::*;

    async fn test_messages() -> Result<(), Error> {
        pretty_env_logger::try_init().ok();
        let user = User::default().with_domain(&crate::domains::domains()?.any().domain);
        let create = create(&user).unwrap();
        let token = crate::token(&user).unwrap();


        let messages = messages(&token.token, None)?;
        assert_eq!(messages.total_items, 0);

        let id = create.id.unwrap();

        accounts::delete(&token.token, &id).unwrap();

        Ok(())
    }

    //TODO other tests
}
