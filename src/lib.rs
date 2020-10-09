pub mod ilo_api_mod {

    use error_chain::error_chain;
    use std::env;
    use serde::Deserialize;

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

    #[derive(Deserialize)]
    pub struct StatusInner {
        Health: String,
        State: String
    }

    #[derive(Deserialize)]
    pub struct Chassis {
        pub Status: StatusInner,
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



        pub fn chassis_status(self) -> Result<String> {
            // Endpoint for gathering Chassis data
            let endpoint: &str = "/redfish/v1/Chassis/1/";
            let full_url: &str = &[&self.url_base, endpoint].join("");

            println!("ASSEMBLED URL: {}, connecting now...", &full_url);

            let res = self.client.get(full_url)
                .header("X-Auth-Token", self.token.to_string())
                .send()?
                .json::<Chassis>()?;

            Ok(res.Status.Health)

        }
    }
}
