#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(unused_assignments)]

mod ai;
mod sql_ops;
mod style;
use ai::*;
use sql_ops::*;
use style::*;

use serde_json::Value;

use std::{error::Error, io::{stdout, self, BufRead}};
use tokio::*;

use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    ExecutableCommand, 
    event,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>>{
    //sqlx::any::install_default_drivers();
    let KEY = fs::read_to_string("logs/api_key.txt").await?;

    let root_system = fs::read_to_string("logs/root_system.txt").await?;
    let error_system = fs::read_to_string("logs/error_system.txt").await?;
    let mut data_assistant = String::new();

    let mut std_handle = io::stdin().lock();
    let mut user_input = String::new();

    let mut bot_binding = DataBot::new(KEY.to_string());
    let bot = bot_binding
        .root_system(root_system)
        .error_system(error_system);

    println!("Hi! Im Data Bot, I can help you explore sqlite databases");
    println!("we need a database connection string to start");

    std_handle.read_line(&mut user_input).unwrap();
    let connection_string = user_input.trim().to_string();
    let db = match DataBase::create_connection(&connection_string).await {
        Ok(db) => {
            _ = styled_println(Color::Green, Color::Reset, "Database Connection Successful")?;
            db
        },
        Err(e) => {
            _ = styled_println(Color::Red, Color::Reset, format!("Database Connection Failed: {}", e).as_str())?;
            return Ok(());
        }
    };

    let db_meta = DataBase::get_database_info(db.clone()).await?;
    println!("{}", db_meta);
    //append data info to data assistant
    data_assistant = format!("the following is information about the database\n{}", db_meta);
    data_assistant.push_str("\nformat your response with the json format specified by your system");

    loop{
        styled_print(Color::Blue, Color::Reset, ">>>")?;

        user_input.clear();
        std_handle.read_line(&mut user_input).unwrap();
        let query = user_input.trim().to_string();

        bot.data_assistant(data_assistant.clone());
        let response = bot.user_query(query).await?;
        //println!("{}", response);
        
        styled_println(Color::Cyan, Color::Reset, response["message"].to_string().as_str())?;
        //query database
        let sql_query = response["sql_query"].to_string()
            .trim_matches('\"')
            //.trim_matches('\'')
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\\\", "\\")
            .to_string();
        let mut query_result = DataBase::query(db.clone(), sql_query.clone(), "fetch".to_string()).await;
        while query_result.is_err(){
            styled_println(Color::Red, Color::Reset, "sql query error, attempting to correct ...")?;
            let correction = bot.query_error(
                format!("the following sql query returns the error: {}", query_result.err().unwrap()),
                sql_query.clone()
            ).await?;
            styled_println(Color::Green, Color::Reset, &correction)?;
            let json:Value = serde_json::from_str(&correction)?;
            let correction = json.get("message").unwrap().to_string()
                .trim_matches('\"')
                //.trim_matches('\'')
                .replace("\\n", "\n")
                .replace("\\t", "\t")
                .replace("\\\\", "\\")
                .to_string();
            query_result = DataBase::query(db.clone(), correction, "fetch".to_string()).await;
        }
        DataBase::pretty_print_data(query_result.unwrap());
    }

    Ok(())
}
