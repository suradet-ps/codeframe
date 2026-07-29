use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn window() -> Option<web_sys::Window> {
  web_sys::window()
}

fn js_call0(obj: &JsValue, method: &str) -> Result<JsValue, JsValue> {
  let func = js_sys::Reflect::get(obj, &JsValue::from_str(method))?;
  let func: js_sys::Function = func.dyn_into()?;
  func.call0(obj)
}

fn js_call1(obj: &JsValue, method: &str, arg: &JsValue) -> Result<JsValue, JsValue> {
  let func = js_sys::Reflect::get(obj, &JsValue::from_str(method))?;
  let func: js_sys::Function = func.dyn_into()?;
  func.call1(obj, arg)
}

/// Register the service worker via raw JS (web-sys lacks ServiceWorker features).
pub fn register_service_worker() {
  let Some(win) = window() else { return };
  let nav = win.navigator();
  let Ok(sw) = js_sys::Reflect::get(&nav.into(), &JsValue::from_str("serviceWorker")) else {
    return;
  };
  if sw.is_undefined() || sw.is_null() {
    return;
  }
  let _ = js_call1(&sw, "register", &JsValue::from_str("/sw.js"));
}

/// PWA install + offline banners — rendered inside Leptos so they survive hydration.
#[component]
pub fn PwaBanners() -> impl IntoView {
  let (install_visible, set_install_visible) = signal(false);
  let (offline_visible, set_offline_visible) = signal(navigator_offline());

  let deferred: RwSignal<Option<JsValue>> = RwSignal::new(None);

  leptos::task::spawn_local(async {
    register_service_worker();
  });

  {
    let set_install = set_install_visible;
    let cb = Closure::wrap(Box::new(move |e: web_sys::Event| {
      e.prevent_default();
      deferred.set(Some(e.unchecked_into()));
      set_install.set(true);
    }) as Box<dyn FnMut(_)>);
    if let Some(win) = window() {
      let _ =
        win.add_event_listener_with_callback("beforeinstallprompt", cb.as_ref().unchecked_ref());
      cb.forget();
    }
  }

  {
    let set_install = set_install_visible;
    let cb = Closure::wrap(Box::new(move |_: web_sys::Event| {
      deferred.set(None);
      set_install.set(false);
    }) as Box<dyn FnMut(_)>);
    if let Some(win) = window() {
      let _ = win.add_event_listener_with_callback("appinstalled", cb.as_ref().unchecked_ref());
      cb.forget();
    }
  }

  {
    let set_offline = set_offline_visible;
    let cb_online = Closure::wrap(Box::new(move |_: web_sys::Event| {
      set_offline.set(false);
    }) as Box<dyn FnMut(_)>);
    let cb_offline = Closure::wrap(Box::new(move |_: web_sys::Event| {
      set_offline.set(true);
    }) as Box<dyn FnMut(_)>);
    if let Some(win) = window() {
      let _ = win.add_event_listener_with_callback("online", cb_online.as_ref().unchecked_ref());
      let _ = win.add_event_listener_with_callback("offline", cb_offline.as_ref().unchecked_ref());
      cb_online.forget();
      cb_offline.forget();
    }
  }

  let install_btn = move |_: web_sys::MouseEvent| {
    let Some(p) = deferred.get() else {
      return;
    };
    let promise = match js_call0(&p, "prompt") {
      Ok(v) => v,
      Err(_) => return,
    };
    let set_install = set_install_visible;
    let then_cb = Closure::once(Box::new(move |_: JsValue| {
      deferred.set(None);
      set_install.set(false);
    }) as Box<dyn FnMut(_)>);
    let _ = js_call1(&promise, "then", then_cb.as_ref().unchecked_ref());
    then_cb.forget();
  };

  let install_dismiss = move |_: web_sys::MouseEvent| {
    deferred.set(None);
    set_install_visible.set(false);
  };

  view! {
      {move || {
          install_visible.get().then(|| {
              view! {
                  <div class="install-banner">
                      <span>"Install CodeFrame for offline use"</span>
                      <button class="install-btn" on:click=install_btn>"Install"</button>
                      <button
                          class="install-dismiss"
                          aria-label="Dismiss install prompt"
                          on:click=install_dismiss
                      >
                          "\u{00d7}"
                      </button>
                  </div>
              }
          })
      }}
      {move || {
          offline_visible.get().then(|| {
              view! {
                  <div class="offline-banner">
                      "You are offline \u{2014} CodeFrame still works."
                  </div>
              }
          })
      }}
  }
}

fn navigator_offline() -> bool {
  window()
    .map(|w| {
      let nav = w.navigator();
      let online = js_sys::Reflect::get(&nav.into(), &JsValue::from_str("onLine"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(true);
      !online
    })
    .unwrap_or(false)
}
