use wasm_bindgen::prelude::*;
use web_sys::{console, WebSocket};
use serde::{Serialize, Deserialize};

#[wasm_bindgen]
pub struct Canvas {
    ws: WebSocket,
}

#[derive(Serialize, Deserialize, Debug)]
struct Update {
    x: usize,
    y: usize,
    color: String,
}

#[wasm_bindgen]
impl Canvas {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Canvas, JsValue> {
        let ws = WebSocket::new("ws://localhost:3030/ws")?;

        let onmessage_callback = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            let data = event.data().as_string().unwrap();
            let update: Update = serde_json::from_str(&data).unwrap();
            console::log_1(&format!("Received update: {:?}", update).into());
        }) as Box<dyn FnMut(web_sys::MessageEvent)>);

        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        Ok(Canvas { ws })
    }

    pub fn update_pixel(&self, x: usize, y: usize, color: String) -> Result<(), JsValue> {
        let update = Update { x, y, color };
        self.ws.send_with_str(&serde_json::to_string(&update).unwrap())?;
        Ok(())
    }
}