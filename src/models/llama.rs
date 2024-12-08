use crate::inference::instance::ChatInstance;

use super::InferenceContext;
use candle_transformers::models::flux::model;

use candle_core::{utils::{cuda_is_available, metal_is_available}, DType, Device};
use candle_nn::VarBuilder;
use tokenizers::Tokenizer;

#[derive(Clone)]
pub struct LLama {

}
impl LLama {
    pub fn new(model_name: String) -> Self{
        Self {

        }
    }
}
impl InferenceContext for LLama {
    // fn load(&mut self, model_name : String)-> anyhow::Result<()> {
    //     todo!()
    // }
    
    fn inference(&mut self, prompt: ChatInstance) -> anyhow::Result<()> {
        println!("inference");
        Ok(())
    }
    // fn load(model_name : String) -> Self{
    // }
    // fn inference() {
        
    // }
    

}