use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use rand::Rng;

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
    Generate,
    GenerateOk {
        id: u32,
    },
}

#[derive(Default)]
struct Node {
    id: u32,
    node_id: String,
    node_ids: Vec<String>,
}

impl Node {
    fn reply(&mut self, input: &Message, payload: Payload) -> Message {
        let msg_id = self.id;
        self.id += 1;
        Message {
            src: input.dst.clone(),
            dst: input.src.clone(),
            body: Body {
                id: Some(msg_id),
                in_reply_to: input.body.id,
                payload,
            },
        }
    }

    pub fn handle(&mut self, input: Message) -> anyhow::Result<Message> {
        match &input.body.payload {
            Payload::Init { node_id, node_ids } => {
                if node_id.is_empty() {
                    bail!("node_id is empty");
                }
                self.node_id = node_id.clone();
                self.node_ids = node_ids.clone();
                Ok(self.reply(&input, Payload::InitOk))
            }
            Payload::Echo { echo } => {
                Ok(self.reply(&input, Payload::EchoOk { echo: echo.clone() }))
            }
            Payload::EchoOk { .. } => bail!("received unexpected EchoOk"),
            Payload::InitOk => bail!("received unexpected InitOk"),
            Payload::Generate => {
                let id = self.generate();
                Ok(self.reply(&input, Payload::GenerateOk { id }))
            }
            Payload::GenerateOk { .. } => bail!("received unexpected GenerateOk"),
        }
    }

    pub fn generate(&mut self) -> u32 {
        let mut rng = rand::rng();
        rng.random::<u32>()
    }
}

pub fn flush(stdout: &mut io::StdoutLock, value: &impl Serialize) -> anyhow::Result<()> {
    serde_json::to_writer(&mut *stdout, value)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let stdin = io::stdin().lock();
    let mut stdout = io::stdout().lock();

    let mut node = Node::default();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message>();

    for input in inputs {
        let reply = node.handle(input?)?;
        flush(&mut stdout, &reply)?;
    }

    Ok(())
}
