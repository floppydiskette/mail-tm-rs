use std::io::Read;
use serde::{Deserialize, Serialize};

use crate::http::{Client};
use crate::user::User;
use crate::MAIL_API_URL;
use anyhow::Error;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub token: String,
    pub id: String,
}

pub(crate) fn token(user: &User) -> Result<Token, Error> {
    let client = Client::new()?
        .build()?;

    log::debug!("Getting token for user {:?}", user);

    let create_as_string = serde_json::json!({
        "address": format!("{}@{}", user.id, user.domain).to_lowercase(),
        "password": user.password
    });

    let mut res = client
        .post(format!("{}/token", MAIL_API_URL.to_owned()).as_str(), create_as_string.to_string())?;

    let body = {
        let mut buffer = String::new();
        res.body_mut().read_to_string(&mut buffer)?;
        buffer
    };
    log::trace!("Retrieved email token: {:?}", body);

    Ok(serde_json::from_str(&body)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::accounts;

    fn test_token() -> Result<(), Error> {
        pretty_env_logger::try_init().ok();
        let user = User::default().with_domain(&crate::domains::domains()?.any().domain);

        let create = accounts::create(&user).unwrap();

        let token = token(&user).unwrap();

        assert_eq!(token.token.is_empty(), false);

        accounts::delete(&token.token, &create.id.unwrap()).unwrap();

        Ok(())
    }
}
