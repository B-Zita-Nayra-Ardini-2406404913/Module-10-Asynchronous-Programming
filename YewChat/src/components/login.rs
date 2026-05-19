use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();

        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| *user.username.borrow_mut() = (*username).clone())
    };

    html! {
        <div class="flex w-screen h-screen items-center justify-center" style="background:#0f172a;">
            <div class="flex flex-col items-center p-10 rounded-2xl w-80" style="background:#1e293b; border:1px solid #334155;">
                <div class="text-5xl mb-2">{"🌐"}</div>
                <h1 class="text-2xl font-bold mb-2" style="color:#38bdf8;">{"Rust WebChat"}</h1>
                <p class="text-sm mb-6 text-center" style="color:#94a3b8;">{"Masukkan username untuk mulai chat"}</p>
                <input
                    type="text"
                    placeholder="Username..."
                    value={(*username).clone()}
                    {oninput}
                    class="w-full py-3 px-4 rounded-xl mb-4 text-sm outline-none"
                    style="background:#0f172a; color:#e2e8f0; border:1px solid #334155;"
                />
                <Link<Route> to={Route::Chat}>
                    <button
                        {onclick}
                        disabled={username.len() < 2}
                        class="w-full py-3 rounded-xl font-bold text-sm"
                        style={
                            if username.len() >= 2 {
                                "background:#38bdf8; color:#0f172a; cursor:pointer; border:none; width:256px;"
                            } else {
                                "background:#334155; color:#64748b; cursor:not-allowed; border:none; width:256px;"
                            }
                        }
                    >
                        { if username.len() >= 2 { "Connect 🚀" } else { "Masukkan username..." } }
                    </button>
                </Link<Route>>
            </div>
        </div>
    }
}