use std::io::Write;

use anyhow::{Error, Result};
use candle_nn::VarBuilder;
use candle_transformers::{generation::LogitsProcessor, models::qwen2::{Config as ConfigBase, ModelForCausalLM as ModelBase}, pipelines::text_generation};
use candle_core::{utils::{cuda_is_available, metal_is_available}, DType, Device, Tensor};
use hf_hub::{api::sync::Api, Repo, RepoType};
use tokenizers::Tokenizer;

use crate::inference::instance::ChatInstance;

use super::{InferenceContext, TokenOutputStream};
#[derive(Clone)]
pub struct Qwen2 {
    model: ModelBase,
    tokenizer: Tokenizer,
    device: Device,
    dtype: DType,
}

impl Qwen2 {
    pub fn new(model_name: String) -> Self {
        let api = Api::new().unwrap();

        let repo = api.repo(Repo::with_revision(
            model_name,
            RepoType::Model,
            "main".to_string(),
        ));
    
        let config_file = repo.get("config.json").unwrap();
        // let tokenizer_filename = repo.get("tokenizer.json").unwrap();
        let tokenizer_filename = repo.get("tokenizer.json").unwrap();
        let filenames = vec![repo.get("model.safetensors").unwrap()];
        
        let device = if cuda_is_available() {
            Device::new_cuda(0).expect("create cuda device failed!")
        } else if metal_is_available() {
            Device::new_metal(0).expect("create metal device failed!")
        } else {
            Device::Cpu
        };
        let dtype = if device.is_cuda() {
            DType::BF16
        } else {
            DType::F32
        };
        
        let vb = unsafe { VarBuilder::from_mmaped_safetensors(&filenames, dtype, &device).unwrap()};
        let model =  {
                let config: ConfigBase = serde_json::from_str(&std::fs::read_to_string(config_file).unwrap()).unwrap();
                ModelBase::new(&config, vb).unwrap()
        };
    
        let tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(Error::msg).unwrap();

        Self {
            model,
            tokenizer,
            device,
            dtype,
        }
    }
}

impl InferenceContext for Qwen2 {
    fn inference(&mut self, prompt: ChatInstance) -> anyhow::Result<()> {
        let mut logits_processor = LogitsProcessor::new(prompt.seed, prompt.temp, prompt.top_p);

        let mut tokenizer = TokenOutputStream::new(self.tokenizer.clone());

        let mut tokens = tokenizer
            .tokenizer()
            .encode(prompt.promote, true)
            .map_err(Error::msg)?
            .get_ids()
            .to_vec();

        std::io::stdout().flush()?;

        let mut generated_tokens = 0usize;
        let eos_token = match tokenizer.get_token("<|endoftext|>") {
            Some(token) => token,
            None => anyhow::bail!("cannot find the <|endoftext|> token"),
        };
        let start_gen = std::time::Instant::now();
        for index in 0..10000 {
            let context_size = if index > 0 { 1 } else { tokens.len() };
            let start_pos = tokens.len().saturating_sub(context_size);
            let ctxt = &tokens[start_pos..];
            let input = Tensor::new(ctxt, &self.device)?.unsqueeze(0)?;
            let logits = self.model.forward(&input, start_pos)?;
            let logits = logits.squeeze(0)?.squeeze(0)?.to_dtype(DType::F32)?;
            let logits = if prompt.repeat_penalty == 1. {
                logits
            } else {
                let start_at = tokens.len().saturating_sub(prompt.repeat_last_n);
                candle_transformers::utils::apply_repeat_penalty(
                    &logits,
                    prompt.repeat_penalty,
                    &tokens[start_at..],
                )?
            };

            let next_token = logits_processor.sample(&logits)?;
            tokens.push(next_token);
            generated_tokens += 1;
            if next_token == eos_token {
                break;
            }
            if let Some(t) = tokenizer.next_token(next_token)? {
                print!("{t}");
                std::io::stdout().flush()?;
                // output.write(format!("{t}")).unwrap();
            }
        }
        // output.end().unwrap();
        let dt = start_gen.elapsed();
        if let Some(rest) = tokenizer.decode_rest().map_err(Error::msg)? {
            print!("{rest}");
        }
        std::io::stdout().flush()?;
        println!(
            "\n{generated_tokens} tokens generated ({:.2} token/s)",
            generated_tokens as f64 / dt.as_secs_f64(),
        );
        Ok(())
    }
}