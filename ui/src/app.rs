use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke_run_bak9_scheduled() -> JsValue;
}

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <main class="container">
            <h1>{"Summary"}</h1>

            <p>{"Last backup: 2024-01-03"}</p>
        </main>
    }
}

