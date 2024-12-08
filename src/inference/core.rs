use std::sync::{Arc, Mutex};

use actix::prelude::*;

use crate::models::InferenceContext;

use super::instance::ChatInstance;

pub struct InferenceActor {
    // inference_context: Box<dyn InferenceContext>,
    inference_context: Arc<Mutex<dyn InferenceContext>>,
}

impl InferenceActor {
    pub fn new(inference_context: Arc<Mutex<dyn InferenceContext>>) -> Self {
        Self {
            inference_context
        }
    }

    pub fn chat_inference(&self, params: ChatInstance) {
        let mut context = self.inference_context.lock().unwrap();
        context.inference(params);
    }
}

impl Actor for InferenceActor {
    type Context = Context<Self>;

    fn started(&mut self,ctx: &mut Self::Context) {
        log::info!("InferenceActor started");
    }
}

#[derive(Message)]
#[rtype(result = "anyhow::Result<InferenceResult>")]
pub enum Openai {
    Chat(ChatInstance),
}

pub enum InferenceResult {
    NULL,
}

impl Handler<Openai> for InferenceActor {
    type Result = anyhow::Result<InferenceResult>;

    fn handle(&mut self, msg: Openai, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            Openai::Chat(instance) => {
                self.chat_inference(instance);
                Ok(InferenceResult::NULL)
            }
        }
    }
}