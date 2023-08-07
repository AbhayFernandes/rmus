use std::fs;

// logs information for now
pub struct TidalSession {
    logged: String,
    client_id: String,
    url: String,
}

impl TidalSession {
    pub fn new(path: String) -> Self {
        let text = fs::read_to_string(path).unwrap();
        let mut lines = text.lines();
        let client_id = lines.next().unwrap().to_string();
        let logged = format!("{}", client_id);
        Self {
            logged,
            client_id,
            url: "https://api.tidal.com/v1/".to_string(),
        }
    }

    fn get_logged_text(&self) -> String {
        self.logged.clone()
    }

    fn login(&mut self) {
        let url = format!("https://auth.tidal.com/v1/oauth2/device_authorization");
        let mut header = reqwest::header::HeaderMap::new();
        header.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        let client = reqwest::blocking::Client::new();
        let response = client
            .post(url)
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("response_type", "code"),
                ("scope", "r_usr w_usr w_sub"),
            ])
            .headers(header)
            .send()
            .unwrap();
        self.logged = format!("response: \n{}", response.text().unwrap());
    }
}
