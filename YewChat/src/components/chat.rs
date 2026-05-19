use serde::{Deserialize, Serialize};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_agent::{Bridge, Bridged};
use crate::services::event_bus::EventBus;
use js_sys;

use crate::{User, services::websocket::WebsocketService};
#[derive(Clone, PartialEq)]
pub enum Msg {
    HandleMsg(String),
    SubmitMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageData {
    pub from: String,
    pub message: String,
    #[serde(default)]
    pub timestamp: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum MsgTypes {
    Users,
    Register,
    Message,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WebSocketMessage {
    message_type: MsgTypes,
    data_array: Option<Vec<String>>,
    data: Option<String>,
}

#[derive(Clone)]
struct UserProfile {
    name: String,
    avatar: String,
}

pub struct Chat {
    users: Vec<UserProfile>,
    chat_input: NodeRef,
    wss: WebsocketService,
    messages: Vec<MessageData>,
    _producer: Box<dyn Bridge<EventBus>>,
}
impl Component for Chat {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let (user, _) = ctx
            .link()
            .context::<User>(Callback::noop())
            .expect("context to be set");
        let wss = WebsocketService::new();
        let username = user.username.borrow().clone();

        let message = WebSocketMessage {
            message_type: MsgTypes::Register,
            data: Some(username.to_string()),
            data_array: None,
        };

        if let Ok(_) = wss
            .tx
            .clone()
            .try_send(serde_json::to_string(&message).unwrap())
        {
            log::debug!("message sent successfully");
        }

        Self {
            users: vec![],
            messages: vec![],
            chat_input: NodeRef::default(),
            wss,
            _producer: EventBus::bridge(ctx.link().callback(Msg::HandleMsg)),
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::HandleMsg(s) => {
                let msg: WebSocketMessage = match serde_json::from_str(&s) {
                    Ok(m) => m,
                    Err(e) => {
                        log::error!("Failed to parse message: {:?}", e);
                        return false;
                    }
                };
                match msg.message_type {
                    MsgTypes::Users => {
                        let users_from_message = msg.data_array.unwrap_or_default();
                        self.users = users_from_message
                            .iter()
                            .map(|u| UserProfile {
                                name: u.into(),
                                avatar: format!(
                                    "https://avatars.dicebear.com/api/adventurer-neutral/{}.svg",
                                    u
                                ),
                            })
                            .collect();
                        true
                    }
                    MsgTypes::Message => {
                        if let Some(data_str) = msg.data {
                            return match serde_json::from_str::<MessageData>(&data_str) {
                                Ok(mut message_data) => {
                                    // Tambahkan timestamp di sisi client
                                    let now = js_sys::Date::new_0();
                                    let hours = now.get_hours();
                                    let minutes = now.get_minutes();
                                    message_data.timestamp = format!("{:02}:{:02}", hours, minutes);
                                    self.messages.push(message_data);
                                    true
                                }
                                Err(e) => {
                                    log::error!("Failed to parse MessageData: {:?}", e);
                                    false
                                }
                            }
                        }
                        false
                    }
                    _ => {
                        false
                    }
                }
            }
            Msg::SubmitMessage => {
                let input = self.chat_input.cast::<HtmlInputElement>();
                if let Some(input) = input {
                    let message = WebSocketMessage {
                        message_type: MsgTypes::Message,
                        data: Some(input.value()),
                        data_array: None,
                    };
                    if let Err(e) = self
                        .wss
                        .tx
                        .clone()
                        .try_send(serde_json::to_string(&message).unwrap())
                    {
                        log::debug!("error sending to channel: {:?}", e);
                    }
                    input.set_value("");
                };
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let submit = ctx.link().callback(|_| Msg::SubmitMessage);
        html! {
        <div class="flex w-screen" style="background:#0f172a; height:100vh;">

            // ===== SIDEBAR KIRI =====
            <div class="flex-none w-64 h-screen" style="background:#1e293b; border-right: 1px solid #334155;">
                <div class="p-4" style="border-bottom: 1px solid #334155;">
                    <div class="text-lg font-bold" style="color:#38bdf8;">{"🌐 Online Users"}</div>
                </div>
                {
                    self.users.clone().iter().map(|u| {
                        html!{
                            <div class="flex items-center m-3 p-3 rounded-xl" style="background:#0f172a;">
                                <img class="w-10 h-10 rounded-full" src={u.avatar.clone()} alt="avatar"/>
                                <div class="ml-3">
                                    <div class="text-sm font-semibold" style="color:#e2e8f0;">{u.name.clone()}</div>
                                    <div class="text-xs" style="color:#38bdf8;">{"● online"}</div>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>

            // ===== PANEL KANAN =====
            <div class="grow h-screen flex flex-col" style="background:#0f172a;">

                // Header
                <div class="w-full h-16 flex items-center px-6" style="background:#1e293b; border-bottom: 1px solid #334155;">
                    <div class="text-xl font-bold" style="color:#38bdf8;">{"💬 Chat Room"}</div>
                </div>

                // Area Pesan
                <div class="w-full grow overflow-auto p-4" style="display:flex; flex-direction:column; gap:12px;">
                    {
                        self.messages.iter().map(|m| {
                            let user = self.users.iter().find(|u| u.name == m.from);
                            let avatar = user.map(|u| u.avatar.clone()).unwrap_or_default();
                            html!{
                                <div class="flex items-start" style="max-width:60%;">
                                    <img class="w-8 h-8 rounded-full mt-1" src={avatar} alt="avatar"/>
                                    <div class="ml-3">
                                        <div class="flex items-center gap-2 mb-1">
                                            <span class="text-sm font-semibold" style="color:#38bdf8;">{m.from.clone()}</span>
                                            <span class="text-xs" style="color:#475569;">{m.timestamp.clone()}</span>
                                        </div>
                                        <div class="rounded-2xl rounded-tl-none px-4 py-2" style="background:#1e293b;">
                                            if m.message.ends_with(".gif") {
                                                <img class="mt-1 rounded-lg" src={m.message.clone()}/>
                                            } else {
                                                <span class="text-sm" style="color:#e2e8f0;">{m.message.clone()}</span>
                                            }
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>

                // Input Area
                <div class="w-full flex items-center px-4 py-3 gap-3" style="background:#1e293b; border-top: 1px solid #334155;">
                    <input
                        ref={self.chat_input.clone()}
                        type="text"
                        placeholder="Ketik pesan..."
                        name="message"
                        required=true
                        class="grow py-3 px-5 rounded-full text-sm outline-none"
                        style="background:#0f172a; color:#e2e8f0; border: 1px solid #334155;"
                    />
                    <button
                        onclick={submit}
                        class="w-11 h-11 rounded-full flex items-center justify-center"
                        style="background:#38bdf8;"
                    >
                        <svg fill="#000000" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg" class="fill-white w-5 h-5">
                            <path d="M0 0h24v24H0z" fill="none"></path>
                            <path d="M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"></path>
                        </svg>
                    </button>
                </div>
            </div>
        </div>
    }
    }
}