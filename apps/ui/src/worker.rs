//! Verbindung zum Satz-Worker (apps/worker, ADR-002).
//!
//! Die Oberfläche enthält kein Typst. Sie startet beim ersten Auftrag den
//! Web Worker und wartet, bis er geladen ist. Jeder Auftrag wird zu einem
//! Promise, das die Antwort mit derselben Kennung erfüllt. Der Worker bleibt
//! für die ganze Sitzung bestehen, auch wenn der Reiter gewechselt wird.

use std::cell::RefCell;
use std::collections::HashMap;

use cc_worker_protocol::{Envelope, Reply, Request, WORKER_SCRIPT};
use js_sys::{Array, Function, Object, Promise, Reflect, Uint8Array};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{ErrorEvent, MessageEvent, Worker};

use crate::engine::js_err;

struct Client {
    worker: Worker,
    /// Erfüllt, sobald der Worker `Ready` meldet.
    ready: Promise,
    next_id: u64,
    /// Offene Aufträge: (resolve, reject) je Kennung
    waiting: HashMap<u64, (Function, Function)>,
}

thread_local! {
    static CLIENT: RefCell<Option<Client>> = const { RefCell::new(None) };
    static READY: RefCell<Option<(Function, Function)>> = const { RefCell::new(None) };
}

fn start() -> Result<Client, String> {
    let worker = Worker::new(WORKER_SCRIPT).map_err(js_err)?;
    let ready = Promise::new(&mut |resolve, reject| {
        READY.with_borrow_mut(|r| *r = Some((resolve, reject)));
    });

    let on_message = Closure::<dyn Fn(MessageEvent)>::new(|ev: MessageEvent| {
        let msg = ev.data();
        let json = Reflect::get(&msg, &"msg".into())
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        let Ok(envelope) = Envelope::<Reply>::from_json(&json) else {
            return;
        };
        if envelope.body == Reply::Ready {
            if let Some((resolve, _)) = READY.with_borrow_mut(Option::take) {
                let _ = resolve.call0(&JsValue::NULL);
            }
            return;
        }
        let waiting =
            CLIENT.with_borrow_mut(|c| c.as_mut().and_then(|c| c.waiting.remove(&envelope.id)));
        if let Some((resolve, _)) = waiting {
            let _ = resolve.call1(&JsValue::NULL, &msg);
        }
    });
    worker.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();

    // Worker nicht ladbar oder abgestürzt: alle Aufträge mit Fehler beenden
    let on_error = Closure::<dyn Fn(ErrorEvent)>::new(|ev: ErrorEvent| {
        let message = JsValue::from_str(&format!("Satz-Worker: {}", ev.message()));
        if let Some((_, reject)) = READY.with_borrow_mut(Option::take) {
            let _ = reject.call1(&JsValue::NULL, &message);
        }
        let waiting: Vec<_> = CLIENT.with_borrow_mut(|c| {
            c.as_mut()
                .map(|c| c.waiting.drain().map(|(_, f)| f).collect())
                .unwrap_or_default()
        });
        for (_, reject) in waiting {
            let _ = reject.call1(&JsValue::NULL, &message);
        }
    });
    worker.set_onerror(Some(on_error.as_ref().unchecked_ref()));
    on_error.forget();

    Ok(Client {
        worker,
        ready,
        next_id: 0,
        waiting: HashMap::new(),
    })
}

/// Schickt einen Auftrag mit Binärdaten und wartet auf die Antwort.
/// `Reply::Error` wird zu `Err`.
pub async fn call(req: Request, data: Vec<Vec<u8>>) -> Result<(Reply, Vec<Vec<u8>>), String> {
    let ready = CLIENT.with_borrow_mut(|c| -> Result<Promise, String> {
        if c.is_none() {
            *c = Some(start()?);
        }
        Ok(c.as_ref().map(|c| c.ready.clone()).expect("gestartet"))
    })?;
    JsFuture::from(ready).await.map_err(js_err)?;

    let answer = CLIENT.with_borrow_mut(|c| -> Result<Promise, String> {
        let c = c.as_mut().ok_or("Satz-Worker nicht gestartet")?;
        c.next_id += 1;
        let id = c.next_id;
        let msg = Object::new();
        let _ = Reflect::set(
            &msg,
            &"msg".into(),
            &Envelope { id, body: req }.to_json().into(),
        );
        let list = Array::new();
        let transfer = Array::new();
        for d in &data {
            let a = Uint8Array::from(d.as_slice());
            transfer.push(&a.buffer());
            list.push(&a);
        }
        let _ = Reflect::set(&msg, &"data".into(), &list);
        let promise = Promise::new(&mut |resolve, reject| {
            c.waiting.insert(id, (resolve, reject));
        });
        if let Err(e) = c.worker.post_message_with_transfer(&msg, &transfer) {
            c.waiting.remove(&id);
            return Err(js_err(e));
        }
        Ok(promise)
    })?;

    let msg = JsFuture::from(answer).await.map_err(js_err)?;
    let json = Reflect::get(&msg, &"msg".into())
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    let reply = Envelope::<Reply>::from_json(&json)?.body;
    if let Reply::Error { message } = reply {
        return Err(message);
    }
    let data = Reflect::get(&msg, &"data".into())
        .ok()
        .filter(|v| v.is_object())
        .map(|v| {
            Array::from(&v)
                .iter()
                .map(|a| Uint8Array::new(&a).to_vec())
                .collect()
        })
        .unwrap_or_default();
    Ok((reply, data))
}
