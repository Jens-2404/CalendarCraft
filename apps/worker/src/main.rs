//! Web Worker, der Kalender mit Typst setzt (ADR-002).
//!
//! Trunk baut dieses Programm getrennt von der Oberfläche (siehe
//! apps/web/index.html, `data-type="worker"`). Die Oberfläche startet den
//! Worker und schickt Aufträge (cc-worker-protocol); das große Typst-Modul
//! lädt so neben der Oberfläche, und das Setzen blockiert sie nicht.

// Nativ nur für Tests gebaut
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
mod satz;

#[cfg(target_arch = "wasm32")]
fn main() {
    use std::cell::RefCell;

    use cc_worker_protocol::{Envelope, Reply, Request};
    use js_sys::{Array, Object, Reflect, Uint8Array};
    use wasm_bindgen::prelude::*;
    use web_sys::{DedicatedWorkerGlobalScope, MessageEvent};

    console_error_panic_hook::set_once();
    let scope: DedicatedWorkerGlobalScope = js_sys::global().unchecked_into();

    // Antwort als `{ msg, data }`; die Puffer werden übergeben, nicht kopiert.
    let post = {
        let scope = scope.clone();
        move |id: u64, body: Reply, data: Vec<Vec<u8>>| {
            let msg = Object::new();
            let _ = Reflect::set(&msg, &"msg".into(), &Envelope { id, body }.to_json().into());
            let list = Array::new();
            let transfer = Array::new();
            for d in data {
                let a = Uint8Array::from(d.as_slice());
                transfer.push(&a.buffer());
                list.push(&a);
            }
            let _ = Reflect::set(&msg, &"data".into(), &list);
            if let Err(e) = scope.post_message_with_transfer(&msg, &transfer) {
                web_sys::console::error_1(&e);
            }
        }
    };

    let satz = RefCell::new(satz::Satz::default());
    let post_reply = post.clone();
    let on_message = Closure::<dyn Fn(MessageEvent)>::new(move |ev: MessageEvent| {
        let msg = ev.data();
        let json = Reflect::get(&msg, &"msg".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let envelope = match Envelope::<Request>::from_json(&json) {
            Ok(e) => e,
            Err(message) => {
                post_reply(0, Reply::Error { message }, Vec::new());
                return;
            }
        };
        let data: Vec<Vec<u8>> = Reflect::get(&msg, &"data".into())
            .ok()
            .filter(|v| v.is_object())
            .map(|v| {
                Array::from(&v)
                    .iter()
                    .map(|a| Uint8Array::new(&a).to_vec())
                    .collect()
            })
            .unwrap_or_default();
        let (reply, out) = satz.borrow_mut().handle(envelope.body, data);
        post_reply(envelope.id, reply, out);
    });
    scope.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    post(0, Reply::Ready, Vec::new());
}

/// Außerhalb des Browsers gibt es nichts zu tun; nativ wird nur `satz`
/// gebaut und getestet.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("cc-worker läuft nur als Web Worker im Browser.");
}
