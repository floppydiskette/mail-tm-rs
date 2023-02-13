use std::io::Read;
use anyhow::Error;
use isahc::ReadResponseExt;
use serde::{Deserialize, Serialize};

use crate::{http, MAIL_API_URL};
use crate::http::Client;
use crate::user::User;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub address: String,
    pub password: Option<String>,
    pub quota: i64,
    pub used: i64,
    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,
    #[serde(rename = "createdAt")]
    pub created_at: serde_json::Value,
    #[serde(rename = "updatedAt")]
    pub updated_at: ::serde_json::Value,
    //TODO these are not values, they're utc date
    #[serde(rename = "@context")]
    pub context: Option<String>,
    #[serde(rename = "@id")]
    pub at_id: Option<String>,
    #[serde(rename = "@type")]
    pub type_field: Option<String>,
    #[serde(rename = "id")]
    pub id: Option<String>,
}

impl Account {
    fn from_user(user: &User) -> Account {
        Account {
            address: format!("{}@{}", user.id, user.domain),
            password: Some(user.password.clone()),
            quota: 0,
            used: 0,
            is_disabled: false,
            created_at: Default::default(),
            updated_at: Default::default(),
            context: None,
            at_id: None,
            type_field: None,
            id: None,
        }
    }
}

pub(crate) fn create(user: &User) -> Result<Account, Error> {
    let client = Client::new()?.build()?;

    log::debug!("Creating account for user {:?}", user);

    let json = serde_json::json!(Account::from_user(user));
    let json_str = json.to_string();
    let mut response = client
        .post(format!("{}/accounts", MAIL_API_URL.to_owned()).as_str(), json_str)?;

    let code = response.status();

    let response_str = {
        let mut buffer = String::new();
        response.body_mut().read_to_string(&mut buffer)?;
        buffer
    };

    http::check_response_status(&code, &response_str)?;

    log::trace!("Created account: {}", response_str);
    Ok(serde_json::from_str(&response_str)?)
}

pub(crate) fn get(token: &str, id: &str) -> Result<Account, Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Searching for account with id {}", id);

    let mut response = client
        .get(&format!("{}/accounts/{}", MAIL_API_URL.to_owned(), id))?;

    let code = response.status();

    let response_str = {
        let mut buffer = String::new();
        response.body_mut().read_to_string(&mut buffer)?;
        buffer
    };

    http::check_response_status(&code, &response_str)?;

    log::trace!("Retrieved a user: {}", response_str);
    Ok(serde_json::from_str(&response_str)?)
}

pub(crate) fn delete(token: &str, id: &str) -> Result<(), Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Searching for account with id {}", id);


    let response = client
        .delete(&format!("{}/accounts/{}", MAIL_API_URL.to_owned(), id))?;

    let code = response.status();

    http::check_response_status(&code, "")?;

    log::trace!("Deleted user with id {}", id);
    Ok(())
}

pub(crate) fn me(token: &str) -> Result<Account, Error> {
    let client = Client::new()?.with_auth(&token)?.build()?;

    log::debug!("Getting me");

    let builder = client
        .get(&format!("{}/me", MAIL_API_URL.to_owned()));

    let mut response = builder?;

    let code = response.status();

    let mut response_str = String::new();
    let n = response.body_mut().read_to_string(&mut response_str)?;

    http::check_response_status(&code, &response_str)?;

    log::trace!("Retrieved me: {}", response_str);
    Ok(serde_json::from_str(&response_str)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token;

    fn test_accounts_create() -> Result<(), Error> {
        pretty_env_logger::try_init().ok();

        let user = User::default().with_domain(&crate::domains::domains()?.any().domain);
        assert_eq!(
            create(&user)?
                .address
                .as_str()
                .is_empty(),
            false
        );
        Ok(())
    }

    async fn test_accounts() -> Result<(), Error> {
        pretty_env_logger::try_init().ok();
        let user = User::default().with_domain(&crate::domains::domains()?.any().domain);

        let create = create(&user).unwrap();

        let token = token(&user).unwrap();


        assert_eq!(
            create
                .address
                .as_str()
                .is_empty(),
            false
        );

        let id = create.id.unwrap();

        let get = get(&token.token, &id)?;

        assert_eq!(get.id.unwrap(), id.clone());

        let me = me(&token.token)?;

        assert_eq!(me.id.unwrap(), id.clone());

        delete(&token.token, &id).unwrap();

        Ok(())
    }
}
