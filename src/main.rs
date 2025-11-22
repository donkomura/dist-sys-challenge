use anyhow::{anyhow, bail};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    src: String,
    #[serde(rename = "dest")]
    dst: String,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Body {
    #[serde(rename = "msg_id")]
    id: Option<u32>,
    in_reply_to: Option<u32>,
    #[serde(flatten)]
    payload: Payload,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum Payload {
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
}

#[derive(Default)]
struct EchoNode {
    id: u32,
    node_id: String,
    node_ids: Vec<String>,
}

impl EchoNode {
    pub fn handle(&mut self, input: Message) -> anyhow::Result<Message> {
        match input.body.payload {
            Payload::Init { node_id, node_ids } => {
                if node_id.is_empty() {
                    return Err(anyhow!("node_id is empty"));
                }
                self.node_id = node_id;
                self.node_ids = node_ids;
                return Ok(Message {
                    src: self.node_id.clone(),
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::InitOk,
                    },
                });
            }
            Payload::Echo { echo } => {
                return Ok(Message {
                    src: self.node_id.clone(),
                    dst: input.src,
                    body: Body {
                        id: Some(self.id),
                        in_reply_to: input.body.id,
                        payload: Payload::EchoOk { echo },
                    },
                });
            }
            Payload::EchoOk { .. } => return Err(anyhow!("unexpected input")),
            Payload::InitOk => bail!("init_ok"),
        };
    }
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut node = EchoNode::default();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();
    for input in inputs {
        let reply = node.handle(input?)?;
        let resp = serde_json::to_string(&reply)?;
        serde_json::to_writer(&mut stdout, &reply);
        stdout.write_all(b"\n");
    }
    Ok(())
}

