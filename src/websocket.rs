use std::collections::HashMap;

use actix::prelude::{Actor, Context, Handler, Message as ActixMessage, Recipient};
use serde::Serialize;
use serde_json::{error::Result as SerdeResult, to_string, Value};
use std::time::{Duration, Instant};
use tracing::{error, info, warn};

use actix::{
    fut,
    prelude::{Addr, StreamHandler},
    ActorContext, ActorFutureExt, AsyncContext, ContextFutureSpawner, WrapFuture,
};
use actix_web_actors::ws;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WebSocketActionType {
    Search,
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(ActixMessage, Serialize)]
#[rtype(result = "()")]
pub struct MessageToClient {
    pub id: Option<String>,
    pub action_type: WebSocketActionType,
    pub data: Value,
}

impl MessageToClient {
    pub fn new(msg_type: WebSocketActionType, data: Value, id: Option<String>) -> Self {
        Self {
            id,
            action_type: msg_type,
            data,
        }
    }
}

pub struct Server {
    sessions: HashMap<String, Recipient<Message>>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    fn send_message_to(&self, id: &str, data: SerdeResult<String>) {
        if let Some(recipient) = self.sessions.get(id) {
            match data {
                Ok(data) => {
                    if let Err(err) = recipient.try_send(Message(data)) {
                        error!("Error sending client message: {:?}", err);
                    }
                }
                Err(err) => {
                    error!("Data did not convert to string {:?}", err);
                }
            }
        } else {
            warn!("No session found with ID: {}", id);
        }
    }

    fn send_message_to_all(&self, data: SerdeResult<String>) {
        match data {
            Ok(data) => {
                for recipient in self.sessions.values() {
                    if let Err(err) = recipient.try_send(Message(data.clone())) {
                        error!("Error sending client message: {:?}", err);
                    }
                }
            }
            Err(err) => {
                error!("Data did not convert to string {:?}", err);
            }
        }
    }
}

impl Actor for Server {
    type Context = Context<Self>;
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Connect {
    pub addr: Recipient<Message>,
    pub id: String,
}

impl Handler<Connect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.id.clone(), msg.addr);
    }
}

#[derive(ActixMessage)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
}

impl Handler<Disconnect> for Server {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.id);
    }
}

impl Handler<MessageToClient> for Server {
    type Result = ();

    fn handle(&mut self, msg: MessageToClient, _: &mut Context<Self>) -> Self::Result {
        let message_str = to_string(&msg);
        if let Some(id) = msg.id {
            self.send_message_to(&id, message_str);
        } else {
            self.send_message_to_all(message_str);
        }
    }
}

pub struct WebSocketSession {
    id: String,
    hb: Instant,
    server_addr: Addr<Server>,
}

impl WebSocketSession {
    pub fn new(key: String, server_addr: Addr<Server>) -> Self {
        Self {
            id: key,
            hb: Instant::now(),
            server_addr,
        }
    }

    fn send_heartbeat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                info!("Websocket Client heartbeat failed, disconnecting!");
                act.server_addr.do_send(Disconnect { id: act.id.clone() });
                // stop actor
                ctx.stop();

                // don't try to send a ping
                return;
            }
            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.send_heartbeat(ctx);

        let session_addr = ctx.address();
        self.server_addr
            .send(Connect {
                addr: session_addr.recipient(),
                id: self.id.clone(),
            })
            .into_actor(self)
            .then(|res, _act, ctx| {
                match res {
                    Ok(_res) => {}
                    _ => ctx.stop(),
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

impl Handler<Message> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            Ok(ws::Message::Close(reason)) => {
                info!("closed ws session");
                self.server_addr.do_send(Disconnect {
                    id: self.id.clone(),
                });
                ctx.close(reason);
                ctx.stop();
            }
            Ok(ws::Message::Text(text)) => {
                // Handle incoming text messages from the user
                info!("Received text message: {}", text);
                // You can process the text message here and optionally send a response
                ctx.text(format!("Echo: {}", text)); // Echo the message back to the client
            }
            Err(err) => {
                warn!("Error handling msg: {:?}", err);
                ctx.stop()
            }
            _ => ctx.stop(),
        }
    }
}
