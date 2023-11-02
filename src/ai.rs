use std::{error::Error, path::PathBuf, io::{self, BufRead}};

use async_openai::{Client, types::{CreateChatCompletionRequestArgs, ChatCompletionRequestMessageArgs, Role, CreateImageRequestArgs, ResponseFormat, ImageSize}, config::OpenAIConfig};
use crossterm::style::Color;
use tiktoken_rs::cl100k_base;
use serde_json::{Value};

use crate::style::styled_println;

#[derive(Clone)]
pub struct Chatbot{
    api_key: String,
    user: String,
    system: String,
    assistant: String,

}

impl Chatbot{
    pub fn new(api_key:String) -> Self{
        Self{
            api_key,
            user: String::new(),
            system: String::new(),
            assistant: String::new(),
        }
    }
    pub fn user(&mut self, user: String) -> &mut Chatbot{
        self.user = user;
        self
    }
    pub fn system(&mut self, system: String) -> &mut Self{
        self.system = system;
        self
    }
    pub fn assistant(&mut self, assistant: String) -> &mut Self{
        self.assistant = assistant;
        self
    }

    pub async fn request(&self, query:String) -> Result<String, Box<dyn Error>>{
        let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(2048u16)
        .model("gpt-3.5-turbo-0613")
        .messages([
            ChatCompletionRequestMessageArgs::default()
                .role(Role::System)
                .content(&self.system)
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::Assistant)
                .content(&self.assistant)
                .build()?,
            ChatCompletionRequestMessageArgs::default()
                .role(Role::User)
                .content(&query)
                .build()?,
        ]).build()?;

        let config = OpenAIConfig::new().with_api_key(self.api_key.clone());
        let client = Client::with_config(config);

        let response = client.chat().create(request).await?;
        let response = &response.choices[0].message.content.as_ref().unwrap();
        Ok(response.to_string())
    }
}

pub struct ImageBot{
    api_key: String,
}

impl ImageBot{
    pub fn new(api_key:String) -> Self{
        Self{
            api_key,
        }
    }

    pub async fn request(&self, query:String, path:String) -> Result<Vec<PathBuf>, Box<dyn Error>>{
        let request = CreateImageRequestArgs::default()
            .prompt(query)
            .response_format(ResponseFormat::B64Json)
            .size(ImageSize::S512x512)
            .build()?;

        let config = OpenAIConfig::new().with_api_key(self.api_key.clone());
        let client = Client::with_config(config);

        let response = client.images().create(request).await?;
        let save = response.save(path).await?;
        Ok(save)
    }
}


pub struct DataBot{
    api_key: String,
    model: String,
    root_system: String,
    error_system: String,
    data_assistant:String,
    conversation_log: Vec<String>
}

impl DataBot{
    pub fn new(api_key:String) -> Self{
        Self{
            api_key,
            model: "gpt-3.5-turbo-0613".to_owned(),
            root_system: String::new(),
            error_system: String::new(),
            data_assistant: String::new(),
            conversation_log: Vec::<String>::new(),
        }
    }

    pub fn root_system(&mut self, system: String) -> &mut Self{
        println!("ROOT SYSTEM ::: ...\n{}", system);
        self.root_system = system;
        self
    }

    pub fn error_system(&mut self, system: String) -> &mut Self{
        println!("ERROR SYSTEM ::: ...\n{}", system);
        self.error_system = system;
        self
    }

    pub fn data_assistant(&mut self, assistant: String) -> &mut Self{
        //println!("DATA ASSISTANT ::: ...\n{}", assistant);
        self.data_assistant = assistant;
        self
    }

    pub fn model(&mut self, model: String) -> &mut Self{
        println!("MODEL ::: ...\n{}", model);
        self.model = model;
        self
    }

    pub fn add_conversation_log(&mut self, string:String) -> &mut Self{
        self.conversation_log.push(string);
        self
    }

    //returns the last x tokens of the conversation log
    pub fn get_conversation_log(&self, tokens:u32) -> String{
        let mut convo_string = String::new();
        let mut logs = self.conversation_log.clone();

        let mut token_count:u32 = 0;
        for log in logs.iter().rev(){
            token_count += cl100k_base().unwrap().encode_ordinary(log).len() as u32;
            if token_count < tokens{
                convo_string.push_str(log);
                convo_string.push_str("\n");
            }
            else{
                break;
            }
        }
        convo_string
    }

    // user query is called on the user's input, 
    //input comming from a bot is handled in another function
    pub async fn user_query(&self, query:String) -> Result<Value, Box<dyn Error>> {
        let modified_query = format!(
"{{
    \"message\": \"{}\",
}}",
            query.trim());
        
        styled_println(
            Color::Green, 
            Color::Reset, 
            format!("Sending User Query:\n{}", modified_query).as_str()
        )?;

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(2048u16)
            .model(self.model.clone())
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(self.root_system.clone())
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(&modified_query)
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::Assistant)
                    .content(&self.data_assistant)
                    .build()?,
            ]).build()?;

        let config = OpenAIConfig::new().with_api_key(self.api_key.clone());
        let client = Client::with_config(config);

        let response = client.chat().create(request).await?;
        styled_println(
            Color::Magenta, 
            Color::Reset, 
            format!("Response: {}", response.choices[0].message.content.as_ref().unwrap()).as_str()
        )?;
        
        //println!("Parcing Json...");
        let mut json:serde_json::Result<Value> = serde_json::from_str(
            response.choices[0].message.content.as_ref().unwrap()
        );
        while json.is_err(){
            styled_println(Color::Red, Color::Reset, "Json Error, attempting to correct ...")?;
            let correction_string = "the following message is not valid json, reformat the response to be valid json";
            let corrected_query = self.query_error(
                correction_string.to_string(), 
                response.choices[0].message.content.as_ref().unwrap().to_string()
            ).await?;
            json = serde_json::from_str(&corrected_query);
        }

        Ok(json.unwrap())
    }

    //function is called when the bot needs to ask the user for more information
    //note: edit the root system to know it can query the user for more information
    //note: edit root system to know it can call this function when it needs to query the user
    pub fn request_info_from_user(&self, query:String) -> String{
        println!(">>> {}", query);
        println!("-v-v-v-");
        //get user input
        let stdin = io::stdin();
        let mut stdin_handle = stdin.lock();

        let mut user_input = String::new();
        stdin_handle.read_line(&mut user_input).unwrap();

        //format user input
        let user_input = format!(
            "{{
                \"message\": \"{}\",
            }}",
            user_input.trim()
        );

        user_input
    }

    // query error is called when the bot sends a response that is not valid json
    pub async fn query_error(&self, error:String, statement:String)-> Result<String, Box<dyn Error>> {
        let modified_query = format!(
            "{{
                \"error\": \"{}\"
                \"message\": \"{}\",
            }}",
            error,
            statement.trim()
        );

        styled_println(
            Color::Red, 
            Color::Reset, 
            format!("Sending Error Query: {}", modified_query).as_str()
        )?;

        let request = CreateChatCompletionRequestArgs::default()
            .max_tokens(2048u16)
            .model(self.model.clone())
            .messages([
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::System)
                    .content(self.error_system.clone())
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::User)
                    .content(&modified_query)
                    .build()?,
                ChatCompletionRequestMessageArgs::default()
                    .role(Role::Assistant)
                    .content(&self.data_assistant)
                    .build()?,
            ]).build()?;
        
        let config = OpenAIConfig::new().with_api_key(self.api_key.clone());
        let client = Client::with_config(config);

        let response = client.chat().create(request).await?;
        println!("Corrected Response: {}", response.choices[0].message.content.as_ref().unwrap());
        Ok(response.choices[0].message.content.as_ref().unwrap().to_string())
    }
}