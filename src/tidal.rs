use crate::ui::Window;
use serde_json::json;
use std::{cell::RefCell, fs, rc::Rc};

pub struct TidalSession {
    client_id: String,
    url: String,
    device_code: String,
    token_type: Option<String>,
    country_code: Option<String>,
    access_token: Option<String>,
    refresh_token: Option<String>,
    log: String,
}

impl TidalSession {
    pub fn save(&self) {
        //remove old tidal_session.json
        let cwd = std::env::current_dir().unwrap();
        let path = cwd.join("tidal_session.json");
        if path.exists() {
            std::fs::remove_file(path).unwrap();
        }
        let json = json!({
            "client_id": self.client_id,
            "url": self.url,
            "device_code": self.device_code,
            "country_code": self.country_code,
            "token_type": self.token_type,
            "access_token": self.access_token,
            "refresh_token": self.refresh_token,
            "log": "",
        });
        let pretty = serde_json::to_string_pretty(&json).unwrap();
        std::fs::write("tidal_session.json", pretty).unwrap();
    }

    pub fn new() -> Self {
        // check if tidal_session.json exists
        let cwd = std::env::current_dir().unwrap();
        let path = cwd.join("tidal_session.json");
        if !path.exists() {
            let text = fs::read_to_string("CREDENTIALS.txt").unwrap();
            let mut lines = text.lines();
            let client_id = lines.next().unwrap().to_string();
            Self {
                client_id,
                device_code: "Empty".to_string(),
                access_token: None,
                token_type: None,
                refresh_token: None,
                country_code: None,
                log: "Empty".to_string(),
                url: "https://api.tidal.com/v1/".to_string(),
            }
        } else {
            // read tidal_session.json
            let tidal_json: serde_json::Value =
                serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
            let client_id = tidal_json["client_id"].to_string();
            let device_code = tidal_json["device_code"].to_string();
            let access_token = tidal_json["access_token"].to_string();
            let refresh_token = tidal_json["refresh_token"].to_string();
            let token_type = tidal_json["token_type"].to_string();
            let country_code = tidal_json["country_code"].to_string();
            // remove quotes from all the above:
            let client_id = client_id[1..client_id.len() - 1].to_string();
            let device_code = device_code[1..device_code.len() - 1].to_string();
            let access_token = access_token[1..access_token.len() - 1].to_string();
            let refresh_token = refresh_token[1..refresh_token.len() - 1].to_string();
            let token_type = token_type[1..token_type.len() - 1].to_string();
            let country_code = country_code[1..country_code.len() - 1].to_string();
            let log = serde_json::to_string_pretty(&tidal_json).unwrap();
            Self {
                client_id,
                device_code,
                country_code: Some(country_code),
                access_token: Some(access_token),
                refresh_token: Some(refresh_token),
                token_type: Some(token_type),
                log,
                url: "https://api.tidal.com/v1/".to_string(),
            }
        }
    }

    pub fn login_oauth(&mut self) {
        // inital request
        self.log = "beginning request".to_string();
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
        let response_text = response.text().unwrap();
        let json: serde_json::Value = serde_json::from_str(response_text.as_str()).unwrap();
        self.device_code = json["deviceCode"].to_string();
        // remove quotes from device code:
        self.device_code = self.device_code[1..self.device_code.len() - 1].to_string();
        let pretty = serde_json::to_string_pretty(&json).unwrap();
        self.log = format!("response: {}\n device code: {}", pretty, self.device_code);
    }

    fn post_after_user(&mut self) -> String {
        let client = reqwest::blocking::Client::new();
        let url = "https://auth.tidal.com/v1/oauth2/token";
        let mut header = reqwest::header::HeaderMap::new();
        header.insert(
            "Content-Type",
            "application/x-www-form-urlencoded".parse().unwrap(),
        );
        let response2 = client
            .post(url)
            .query(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_id.as_str()),
                ("device_code", self.device_code.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                ("scope", "r_usr"),
            ])
            .headers(header)
            .send()
            .unwrap();
        if response2.status().is_success() {
            let json: serde_json::Value = serde_json::from_str(&response2.text().unwrap()).unwrap();
            let access_token = json["access_token"].to_string();
            let refresh_token = json["refresh_token"].to_string();
            let token_type = json["token_type"].to_string();
            let country_code = json["countryCode"].to_string();
            // remove quotes from both tokens:
            self.access_token = Some(access_token[1..access_token.len() - 1].to_string());
            self.refresh_token = Some(refresh_token[1..refresh_token.len() - 1].to_string());
            self.token_type = Some(token_type[1..token_type.len() - 1].to_string());
            self.country_code = Some(country_code[1..country_code.len() - 1].to_string());
            serde_json::to_string_pretty(&json).unwrap()
        } else {
            format!("{}\n\n{}", response2.status(), response2.text().unwrap())
        }
    }
}

pub struct TidalWindow {
    pub session: Rc<RefCell<TidalSession>>,
    title: String,
}

impl Window for TidalWindow {
    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn draw(
        &mut self,
        area: tui::prelude::Rect,
        f: &mut tui::Frame<tui::prelude::CrosstermBackend<std::io::Stdout>>,
    ) -> Result<(), std::io::Error> {
        let output = tui::widgets::Paragraph::new(self.session.borrow().log.clone());
        f.render_widget(output, area);
        Ok(())
    }

    fn handle_input(&mut self, key: crossterm::event::KeyCode) -> Result<(), std::io::Error> {
        match key {
            crossterm::event::KeyCode::Char('q') => {
                std::process::exit(0);
            }
            crossterm::event::KeyCode::Char('e') => {
                self.session.borrow_mut().login_oauth();
            }
            crossterm::event::KeyCode::Char('f') => {
                let mut session = self.session.borrow_mut();
                session.log = session.post_after_user();
            }
            _ => {}
        }
        Ok(())
    }
}

impl TidalWindow {
    pub fn new(session: Rc<RefCell<TidalSession>>) -> Self {
        Self {
            session,
            title: "Tidal".to_string(),
        }
    }
}
