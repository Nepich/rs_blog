use gloo_net::http::Request;
use serde_json::{json, Value};
use serde_wasm_bindgen::to_value;
use wasm_bindgen::prelude::*;
use web_sys::{window, Storage};

const STORAGE_TOKEN_KEY: &str = "blog_token";
const STORAGE_USERNAME_KEY: &str = "blog_username";
const STORAGE_USER_ID_KEY: &str = "blog_user_id";
const DEFAULT_SERVER_URL: &str = "http://localhost:3000/api";

#[wasm_bindgen]
pub struct BlogApp {
    server_url: String,
    token: Option<String>,
    username: Option<String>,
    user_id: Option<i64>,
}

#[wasm_bindgen]
impl BlogApp {
    #[wasm_bindgen(constructor)]
    pub fn new() -> BlogApp {
        console_error_panic_hook::set_once();

        let mut app = BlogApp {
            server_url: DEFAULT_SERVER_URL.to_string(),
            token: None,
            username: None,
            user_id: None,
        };

        if let Ok(Some(token)) = app.get_token_from_storage() {
            app.token = Some(token);
        }

        if let Ok(Some(username)) = app.get_username_from_storage() {
            app.username = Some(username);
        }

        if let Ok(Some(user_id)) = app.get_user_id_from_storage() {
            app.user_id = Some(user_id);
        }

        app
    }

    #[wasm_bindgen]
    pub fn is_authenticated(&self) -> bool {
        self.token.is_some()
    }

    #[wasm_bindgen]
    pub fn current_username(&self) -> String {
        self.username.clone().unwrap_or_default()
    }

    #[wasm_bindgen]
    pub fn current_user_id(&self) -> JsValue {
        match self.user_id {
            Some(id) => JsValue::from_f64(id as f64),
            None => JsValue::NULL,
        }
    }

    #[wasm_bindgen]
    pub fn set_server_url(&mut self, url: String) -> Result<(), JsValue> {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return Err(JsValue::from_str("Server URL cannot be empty"));
        }

        self.server_url = trimmed.trim_end_matches('/').to_string();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn logout(&mut self) -> Result<JsValue, JsValue> {
        self.token = None;
        self.username = None;
        self.user_id = None;
        self.clear_storage()?;
        Ok(to_value(&json!({ "success": true, "message": "Logged out" }))?)
    }

    #[wasm_bindgen]
    pub async fn register(
        &mut self,
        username: String,
        email: String,
        password: String,
    ) -> Result<JsValue, JsValue> {
        if username.trim().is_empty() || email.trim().is_empty() || password.trim().is_empty() {
            return Err(JsValue::from_str(
                "Username, email and password cannot be empty",
            ));
        }

        let body = json!({
            "username": username,
            "email": email,
            "password": password,
        });

        let response = self
            .send_request("auth/register", "POST", Some(body), false)
            .await?;

        let token = response
            .get("token")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| JsValue::from_str("Server did not return a token"))?;

        let user = response
            .get("user")
            .and_then(Value::as_object)
            .ok_or_else(|| JsValue::from_str("Server did not return user data"))?;

        let username = user
            .get("username")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| JsValue::from_str("User data did not include username"))?;

        let user_id = user
            .get("id")
            .and_then(Value::as_i64)
            .ok_or_else(|| JsValue::from_str("User data did not include id"))?;

        self.save_login(token.clone(), username.clone(), user_id)?;
        Ok(to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn login(&mut self, username: String, password: String) -> Result<JsValue, JsValue> {
        if username.trim().is_empty() || password.trim().is_empty() {
            return Err(JsValue::from_str("Username and password cannot be empty"));
        }

        let body = json!({
            "username": username,
            "password": password,
        });

        let response = self
            .send_request("auth/login", "POST", Some(body), false)
            .await?;

        let token = response
            .get("token")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| JsValue::from_str("Server did not return a token"))?;

        let user = response
            .get("user")
            .and_then(Value::as_object)
            .ok_or_else(|| JsValue::from_str("Server did not return user data"))?;

        let username = user
            .get("username")
            .and_then(Value::as_str)
            .map(String::from)
            .ok_or_else(|| JsValue::from_str("User data did not include username"))?;

        let user_id = user
            .get("id")
            .and_then(Value::as_i64)
            .ok_or_else(|| JsValue::from_str("User data did not include id"))?;

        self.save_login(token.clone(), username.clone(), user_id)?;
        Ok(to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn load_posts(&self) -> Result<JsValue, JsValue> {
        let response = self
            .send_request("posts?limit=100", "GET", None, false)
            .await?;

        let posts = response.get("posts").cloned().unwrap_or_else(|| json!([]));
        Ok(to_value(&posts)?)
    }

    #[wasm_bindgen]
    pub async fn create_post(&mut self, title: String, content: String) -> Result<JsValue, JsValue> {
        if title.trim().is_empty() || content.trim().is_empty() {
            return Err(JsValue::from_str("Title and content cannot be empty"));
        }

        let body = json!({
            "title": title,
            "content": content,
        });

        let response = self
            .send_request("posts", "POST", Some(body), true)
            .await?;
        Ok(to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn update_post(
        &mut self,
        id: u64,
        title: String,
        content: String,
    ) -> Result<JsValue, JsValue> {
        if title.trim().is_empty() || content.trim().is_empty() {
            return Err(JsValue::from_str("Title and content cannot be empty"));
        }

        let body = json!({
            "title": title,
            "content": content,
        });

        let response = self
            .send_request(&format!("posts/{}", id), "PUT", Some(body), true)
            .await?;
        Ok(to_value(&response)?)
    }

    #[wasm_bindgen]
    pub async fn delete_post(&mut self, id: u64) -> Result<JsValue, JsValue> {
        let response = self
            .send_request(&format!("posts/{}", id), "DELETE", None, true)
            .await?;
        Ok(to_value(&response)?)
    }
}

impl BlogApp {
    fn local_storage(&self) -> Result<Storage, JsValue> {
        let window = window().ok_or_else(|| JsValue::from_str("Unable to access window"))?;
        let storage = window
            .local_storage()
            .map_err(|_| JsValue::from_str("Unable to access localStorage"))?
            .ok_or_else(|| JsValue::from_str("localStorage is not available"))?;
        Ok(storage)
    }

    fn save_str_to_storage(&self, key: &str, value: &str) -> Result<(), JsValue> {
        let storage = self.local_storage()?;
        storage
            .set_item(key, value)
            .map_err(|_| JsValue::from_str("Failed to save item to localStorage"))
    }

    fn get_str_from_storage(&self, key: &str) -> Result<Option<String>, JsValue> {
        let storage = self.local_storage()?;
        storage
            .get_item(key)
            .map_err(|_| JsValue::from_str("Failed to read item from localStorage"))
    }

    fn clear_storage(&self) -> Result<(), JsValue> {
        let storage = self.local_storage()?;
        storage
            .remove_item(STORAGE_TOKEN_KEY)
            .map_err(|_| JsValue::from_str("Failed to clear token from localStorage"))?;
        storage
            .remove_item(STORAGE_USERNAME_KEY)
            .map_err(|_| JsValue::from_str("Failed to clear username from localStorage"))?;
        storage
            .remove_item(STORAGE_USER_ID_KEY)
            .map_err(|_| JsValue::from_str("Failed to clear user ID from localStorage"))?;
        Ok(())
    }

    fn save_login(&mut self, token: String, username: String, user_id: i64) -> Result<(), JsValue> {
        self.token = Some(token.clone());
        self.username = Some(username.clone());
        self.user_id = Some(user_id);
        self.save_str_to_storage(STORAGE_TOKEN_KEY, &token)?;
        self.save_str_to_storage(STORAGE_USERNAME_KEY, &username)?;
        self.save_str_to_storage(STORAGE_USER_ID_KEY, &user_id.to_string())?;
        Ok(())
    }

    fn get_token_from_storage(&self) -> Result<Option<String>, JsValue> {
        self.get_str_from_storage(STORAGE_TOKEN_KEY)
    }

    fn get_username_from_storage(&self) -> Result<Option<String>, JsValue> {
        self.get_str_from_storage(STORAGE_USERNAME_KEY)
    }

    fn get_user_id_from_storage(&self) -> Result<Option<i64>, JsValue> {
        match self.get_str_from_storage(STORAGE_USER_ID_KEY)? {
            Some(value) => value
                .parse::<i64>()
                .map(Some)
                .map_err(|_| JsValue::from_str("Failed to parse stored user ID")),
            None => Ok(None),
        }
    }

    async fn send_request(
        &self,
        path: &str,
        method: &str,
        body: Option<Value>,
        auth: bool,
    ) -> Result<Value, JsValue> {
        let url = format!(
            "{}/{}",
            self.server_url.trim_end_matches('/'),
            path.trim_start_matches('/'),
        );

        let mut request = match method {
            "POST" => Request::post(&url),
            "PUT" => Request::put(&url),
            "DELETE" => Request::delete(&url),
            _ => Request::get(&url),
        };

        if auth {
            let token = self
                .token
                .as_ref()
                .ok_or_else(|| JsValue::from_str("Authentication required"))?;
            request = request.header("Authorization", &format!("Bearer {}", token));
        }

        let response = if let Some(body_value) = body {
            request
                .header("Content-Type", "application/json")
                .json(&body_value)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize request body: {}", e)))?
                .send()
                .await
                .map_err(|e| JsValue::from_str(&format!("Network request failed: {}", e)))?
        } else {
            request
                .send()
                .await
                .map_err(|e| JsValue::from_str(&format!("Network request failed: {}", e)))?
        };

        if !response.ok() {
            let text = response
                .text()
                .await
                .map_err(|e| JsValue::from_str(&format!("Failed to read error response: {}", e)))?;

            let details = if text.trim().is_empty() {
                json!({ "status": response.status(), "message": "Request failed" })
            } else if let Ok(json_value) = serde_json::from_str::<Value>(&text) {
                json_value
            } else {
                json!({ "status": response.status(), "message": text })
            };

            return Err(to_value(&details)?);
        }

        let text = response
            .text()
            .await
            .map_err(|e| JsValue::from_str(&format!("Failed to read response: {}", e)))?;

        if text.trim().is_empty() {
            return Ok(json!({ "success": true }));
        }

        serde_json::from_str(&text)
            .map_err(|e| JsValue::from_str(&format!("Malformed JSON response: {}", e)))
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod tests {
    use super::*;

    #[test]
    fn test_local_storage_helpers_do_not_crash() {
        let app = BlogApp {
            server_url: DEFAULT_SERVER_URL.to_string(),
            token: Some(String::from("abc")),
            username: Some(String::from("user")),
            user_id: Some(1),
        };
        let _ = app.get_token_from_storage();
        let _ = app.get_username_from_storage();
        let _ = app.get_user_id_from_storage();
    }

    #[test]
    fn test_set_server_url_trims_slashes() {
        let mut app = BlogApp {
            server_url: DEFAULT_SERVER_URL.to_string(),
            token: None,
            username: None,
            user_id: None,
        };
        app.set_server_url("http://example.com/".to_string())
            .expect("failed to set server URL");
        assert_eq!(app.server_url, "http://example.com");
    }

    #[test]
    fn test_set_server_url_rejects_empty_string() {
        let mut app = BlogApp {
            server_url: DEFAULT_SERVER_URL.to_string(),
            token: None,
            username: None,
            user_id: None,
        };
        assert!(app.set_server_url("".to_string()).is_err());
    }

    #[test]
    fn test_auth_state_helpers() {
        let app = BlogApp {
            server_url: DEFAULT_SERVER_URL.to_string(),
            token: Some("token".to_string()),
            username: Some("bob".to_string()),
            user_id: Some(99),
        };

        assert!(app.is_authenticated());
        assert_eq!(app.current_username(), "bob");
        assert_eq!(app.current_user_id(), JsValue::from_f64(99.0));
    }
}
