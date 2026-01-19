//! WebSocket client for real-time communication

use gloo_storage::{LocalStorage, Storage};
use shared::dto::WebSocketMessage;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, MessageEvent, WebSocket};

const WS_URL: &str = "ws://localhost:8080/ws";
const TOKEN_KEY: &str = "rust_teams_token";

pub struct WebSocketClient {
    ws: Rc<RefCell<Option<WebSocket>>>,
    on_message: Rc<RefCell<Option<Box<dyn Fn(WebSocketMessage)>>>>,
    on_connect: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_disconnect: Rc<RefCell<Option<Box<dyn Fn()>>>>,
    on_error: Rc<RefCell<Option<Box<dyn Fn(String)>>>>,
}

impl WebSocketClient {
    pub fn new() -> Self {
        Self {
            ws: Rc::new(RefCell::new(None)),
            on_message: Rc::new(RefCell::new(None)),
            on_connect: Rc::new(RefCell::new(None)),
            on_disconnect: Rc::new(RefCell::new(None)),
            on_error: Rc::new(RefCell::new(None)),
        }
    }

    pub fn connect(&self) -> Result<(), String> {
        let ws = WebSocket::new(WS_URL).map_err(|e| format!("{:?}", e))?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Set up event handlers
        let on_message = self.on_message.clone();
        let onmessage_callback = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                if let Ok(msg) = serde_json::from_str::<WebSocketMessage>(&text) {
                    if let Some(callback) = on_message.borrow().as_ref() {
                        callback(msg);
                    }
                }
            }
        });
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        let on_connect = self.on_connect.clone();
        let ws_clone = ws.clone();
        let onopen_callback = Closure::<dyn FnMut()>::new(move || {
            log::info!("WebSocket connected");

            // Authenticate with token
            if let Ok(token) = LocalStorage::get::<String>(TOKEN_KEY) {
                let auth_msg = WebSocketMessage::Authenticate { token };
                if let Ok(json) = serde_json::to_string(&auth_msg) {
                    let _ = ws_clone.send_with_str(&json);
                }
            }

            if let Some(callback) = on_connect.borrow().as_ref() {
                callback();
            }
        });
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();

        let on_disconnect = self.on_disconnect.clone();
        let onclose_callback = Closure::<dyn FnMut(_)>::new(move |_: CloseEvent| {
            log::info!("WebSocket disconnected");
            if let Some(callback) = on_disconnect.borrow().as_ref() {
                callback();
            }
        });
        ws.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();

        let on_error = self.on_error.clone();
        let onerror_callback = Closure::<dyn FnMut(_)>::new(move |e: web_sys::ErrorEvent| {
            let error_msg = e.message();
            log::error!("WebSocket error: {}", error_msg);
            if let Some(callback) = on_error.borrow().as_ref() {
                callback(error_msg);
            }
        });
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();

        *self.ws.borrow_mut() = Some(ws);
        Ok(())
    }

    pub fn disconnect(&self) {
        if let Some(ws) = self.ws.borrow().as_ref() {
            let _ = ws.close();
        }
        *self.ws.borrow_mut() = None;
    }

    pub fn send(&self, message: &WebSocketMessage) -> Result<(), String> {
        if let Some(ws) = self.ws.borrow().as_ref() {
            let json = serde_json::to_string(message).map_err(|e| e.to_string())?;
            ws.send_with_str(&json).map_err(|e| format!("{:?}", e))?;
            Ok(())
        } else {
            Err("WebSocket not connected".to_string())
        }
    }

    pub fn is_connected(&self) -> bool {
        self.ws
            .borrow()
            .as_ref()
            .map(|ws| ws.ready_state() == WebSocket::OPEN)
            .unwrap_or(false)
    }

    pub fn on_message<F: Fn(WebSocketMessage) + 'static>(&self, callback: F) {
        *self.on_message.borrow_mut() = Some(Box::new(callback));
    }

    pub fn on_connect<F: Fn() + 'static>(&self, callback: F) {
        *self.on_connect.borrow_mut() = Some(Box::new(callback));
    }

    pub fn on_disconnect<F: Fn() + 'static>(&self, callback: F) {
        *self.on_disconnect.borrow_mut() = Some(Box::new(callback));
    }

    pub fn on_error<F: Fn(String) + 'static>(&self, callback: F) {
        *self.on_error.borrow_mut() = Some(Box::new(callback));
    }

    // Convenience methods
    pub fn join_channel(&self, channel_id: uuid::Uuid) -> Result<(), String> {
        self.send(&WebSocketMessage::JoinChannel { channel_id })
    }

    pub fn leave_channel(&self, channel_id: uuid::Uuid) -> Result<(), String> {
        self.send(&WebSocketMessage::LeaveChannel { channel_id })
    }

    pub fn send_message(
        &self,
        channel_id: uuid::Uuid,
        content: String,
        reply_to_id: Option<uuid::Uuid>,
    ) -> Result<(), String> {
        self.send(&WebSocketMessage::SendMessage {
            channel_id,
            content,
            reply_to_id,
        })
    }

    pub fn start_typing(&self, channel_id: uuid::Uuid) -> Result<(), String> {
        self.send(&WebSocketMessage::StartTyping { channel_id })
    }

    pub fn stop_typing(&self, channel_id: uuid::Uuid) -> Result<(), String> {
        self.send(&WebSocketMessage::StopTyping { channel_id })
    }

    pub fn update_status(
        &self,
        status: shared::models::UserStatus,
        status_message: Option<String>,
    ) -> Result<(), String> {
        self.send(&WebSocketMessage::UpdateStatus {
            status,
            status_message,
        })
    }
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self::new()
    }
}
