pub mod ilo_api_mod {

    use error_chain::error_chain;
    use std::env;
    use serde::{ Deserialize, Serialize };

    error_chain! {
        foreign_links {
            EnvVar(env::VarError);
            HttpRequest(reqwest::Error);
            MalformedToken(reqwest::header::ToStrError);
        }

        errors {
            MissingTokenHeader
        }
    }

    pub struct IloSession {
        pub url_base: String,
        pub token: String,
        pub client: reqwest::blocking::Client
    }

    #[derive(Debug)]
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Root {
        #[serde(rename = "Status")]
        pub status: Status2,
        #[serde(rename = "Oem")]
        pub oem: Oem,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Oem {
        #[serde(rename = "Hpe")]
        pub hpe: Hpe,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Hpe {
        #[serde(rename = "SmartStorageBattery")]
        pub smart_storage_battery: Vec<SmartStorageBattery>,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status2 {
        #[serde(rename = "Health")]
        pub health: String,
        #[serde(rename = "State")]
        pub state: String,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct SmartStorageBattery {
        #[serde(rename = "Status")]
        pub status: Status,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Status {
        #[serde(rename = "Health")]
        pub health: String,
        #[serde(rename = "State")]
        pub state: String,
    }

    impl IloSession {
        pub fn create (url: &str, user: &str, passw: &str) -> Result<Self> {
            let endpoint: &str = "/redfish/v1/SessionService/Sessions/";
            let full_url: &str = &[&url, endpoint].join("");

            println!("ASSEMBLED URL: {}, connecting now...", &full_url);

            let auth_body = serde_json::json!({
                "UserName": user,
                "Password": passw
            });

            let client = reqwest::blocking::Client::builder()
                .danger_accept_invalid_certs(true)
                .build()?;

            let res = client.post(full_url).json(&auth_body).send()?;

            let token = res
                .headers()
                .get("x-auth-token")
                .ok_or(ErrorKind::MissingTokenHeader)?
                .to_str()?
                .to_owned();

            Ok(Self { token, client, url_base: url.to_owned() })
        }

        pub fn chassis(self) -> Result<Root> {
            // Endpoint for gathering Chassis data
            let endpoint: &str = "/redfish/v1/Chassis/1/";
            let full_url: &str = &[&self.url_base, endpoint].join("");

            println!("ASSEMBLED URL: {}, connecting now...", &full_url);

            let res = self.client.get(full_url)
                .header("X-Auth-Token", self.token.to_string())
                .send()?
                .json::<Root>()?;

            Ok(res)
        }
    }
}
